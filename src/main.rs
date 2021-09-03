#![feature(maybe_uninit_extra)]
#![feature(const_panic)]

mod cpu;
mod device;
#[cfg(test)]
mod tests;
mod types;

use cpu::Cpu;
use device::{Audio, Lcd, Uart};
use types::*;

use std::collections::VecDeque;
use std::io::{Stdout, Write};

use crossterm::cursor::*;
use crossterm::style::*;
use crossterm::terminal::*;
use crossterm::{ExecutableCommand, QueueableCommand};
use ggez::conf::{NumSamples, WindowMode, WindowSetup};
use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics::{
    Color, DrawParam, FilterMode, Font, Image, PxScale, Text, TextFragment, WrapMode,
};
use ggez::{event, graphics, timer, Context, ContextBuilder, GameError, GameResult};
use spin_sleep::LoopHelper;

const TITLE: &str = "Pipelined CPU Emu";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

const CLOCK_RATE: f64 = 1_000_000.0; // 1 MHz
const FRAME_RATE: u32 = 60;
const CYCLES_PER_FRAME: f64 = CLOCK_RATE / (FRAME_RATE as f64);
const WHOLE_CYCLES_PER_FRAME: u64 = CYCLES_PER_FRAME as u64;
const FRACT_CYCLES_PER_FRAME: f64 = CYCLES_PER_FRAME - (WHOLE_CYCLES_PER_FRAME as f64);

const UART_BAUD_RATE: f64 = 115_200.0; // 115.2 kHz
const CYCLES_PER_BAUD: f64 = CLOCK_RATE / UART_BAUD_RATE;

fn format_clock_rate(clock_rate: f64) -> String {
    if clock_rate > 999_000_000.0 {
        format!("{:.1} GHz", clock_rate / 1_000_000_000.0)
    } else if clock_rate > 999_000.0 {
        format!("{:.1} MHz", clock_rate / 1_000_000.0)
    } else if clock_rate > 999.0 {
        format!("{:.1} kHz", clock_rate / 1_000.0)
    } else {
        format!("{:.0} Hz", clock_rate)
    }
}

pub struct Memory {
    data: [u8; 0x10000],
}
impl Memory {
    #[inline]
    pub const fn new() -> Self {
        Self { data: [0; 0x10000] }
    }

    #[inline]
    pub fn create() -> Box<Self> {
        Box::new(Self::new())
    }

    pub fn from_rom(rom: &[u8]) -> Box<Self> {
        let mut data = [0; 0x10000];
        data[0..rom.len()].copy_from_slice(rom);
        Box::new(Self { data })
    }

    #[inline]
    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    #[inline]
    pub fn write(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value;
    }
}

struct Utf8Builder {
    byte_count: usize,
    bytes: Vec<u8>,
}
impl Utf8Builder {
    pub fn new(first_byte: u8) -> Self {
        let byte_count = if (first_byte & 0b11111000) == 0b11110000 {
            4
        } else if (first_byte & 0b11110000) == 0b11100000 {
            3
        } else {
            2
        };

        let mut bytes = Vec::with_capacity(byte_count);
        bytes.push(first_byte);

        Self { byte_count, bytes }
    }

    pub fn process_data(&mut self, data: u8) -> Option<char> {
        self.bytes.push(data);

        if self.bytes.len() == self.byte_count {
            if let Ok(s) = String::from_utf8(self.bytes.clone()) {
                let chars: Vec<char> = s.chars().collect();
                if chars.len() > 0 {
                    Some(chars[0])
                } else {
                    Some(char::REPLACEMENT_CHARACTER)
                }
            } else {
                Some(char::REPLACEMENT_CHARACTER)
            }
        } else {
            None
        }
    }
}

struct EmuState {
    cpu: Cpu,
    memory: Box<Memory>,
    lcd: Lcd,
    uart: Uart,
    audio: Audio,

    running: bool,
    fractional_cycles: f64,
    baud_cycles: f64,
    loop_helper: LoopHelper,

    stdout: Stdout,
    input_queue: VecDeque<u8>,
    output_queue: VecDeque<u8>,
    partial_char: Option<Utf8Builder>,

    font: Font,
    show_debug_info: bool,
}
impl EmuState {
    pub fn create(font: Font) -> GameResult<Self> {
        const ROM_BYTES: &[u8] = include_bytes!("../res/snek.bin");
        //const ROM_BYTES: &[u8] = include_bytes!("../res/rom.bin");

        let mut stdout = std::io::stdout();
        stdout.execute(Clear(ClearType::All))?;
        stdout.execute(MoveTo(0, 0))?;

        Ok(Self {
            cpu: Cpu::new(),
            memory: Memory::from_rom(ROM_BYTES),
            lcd: Lcd::new(),
            uart: Uart::new(),
            audio: Audio::new(),

            running: false,
            fractional_cycles: 0.0,
            baud_cycles: 0.0,
            loop_helper: LoopHelper::builder()
                .native_accuracy_ns(1_500_000)
                .report_interval_s(0.5)
                .build_with_target_rate(FRAME_RATE),

            stdout,
            input_queue: VecDeque::new(),
            output_queue: VecDeque::new(),
            partial_char: None,

            font,
            show_debug_info: true,
        })
    }

    #[inline]
    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    fn process_uart(&mut self) -> GameResult {
        if let Some(data) = self.uart.host_read() {
            self.output_queue.push_back(data);
        }

        if let Some(data) = self.input_queue.pop_front() {
            self.uart.host_write(data);
        }

        Ok(())
    }

    fn process_terminal(&mut self) -> GameResult {
        while let Some(data) = self.output_queue.pop_front() {
            if let Some(high_bytes) = &mut self.partial_char {
                if let Some(c) = high_bytes.process_data(data) {
                    self.stdout.queue(Print(c))?;
                    self.partial_char = None;
                }
            } else {
                if (data & 0x80) == 0 {
                    let c = char::from(data);
                    self.stdout.queue(Print(c))?;
                } else {
                    self.partial_char = Some(Utf8Builder::new(data));
                }
            }
        }

        Ok(())
    }

    fn clock(&mut self) -> GameResult<bool> {
        let break_point = self.cpu.clock(
            self.memory.as_mut(),
            &mut self.lcd,
            &mut self.uart,
            &mut self.audio,
        );

        self.baud_cycles += 1.0;
        while self.baud_cycles >= CYCLES_PER_BAUD {
            self.baud_cycles -= CYCLES_PER_BAUD;

            self.process_uart()?;
        }

        Ok(break_point)
    }

    fn single_clock(&mut self) -> GameResult<bool> {
        let break_point = self.clock()?;

        self.process_terminal()?;
        self.stdout.flush()?;

        Ok(break_point)
    }
}
impl EventHandler<GameError> for EmuState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.loop_helper.loop_sleep();
        self.loop_helper.loop_start();

        if let Some(fps) = self.loop_helper.report_rate() {
            if self.running {
                graphics::set_window_title(
                    ctx,
                    &format!(
                        "{} v{} - {:.1} fps - {}",
                        TITLE,
                        VERSION,
                        fps,
                        format_clock_rate(fps * CYCLES_PER_FRAME)
                    ),
                );
            } else {
                graphics::set_window_title(
                    ctx,
                    &format!("{} v{} - {:.1} fps", TITLE, VERSION, fps),
                );
            }
        }

        while timer::check_update_time(ctx, FRAME_RATE) {
            if self.running {
                self.fractional_cycles += FRACT_CYCLES_PER_FRAME;
                let cycles_to_add = self.fractional_cycles as u64;
                self.fractional_cycles -= cycles_to_add as f64;
                let cycle_count = WHOLE_CYCLES_PER_FRAME + cycles_to_add;

                for _ in 0..cycle_count {
                    if self.clock()? {
                        self.running = false;
                        break;
                    }
                }
            }
        }

        self.process_terminal()?;
        self.stdout.flush()?;

        timer::yield_now();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);

        // ToDo: VGA

        if self.show_debug_info {
            const TEXT_SCALE: PxScale = PxScale { x: 20.0, y: 20.0 };
            const TEXT_BACK_COLOR: graphics::Color = graphics::Color::new(0.0, 0.0, 0.0, 1.0);
            const TEXT_FRONT_COLOR: graphics::Color = graphics::Color::new(0.5, 1.0, 0.0, 1.0);

            let cpu_info = format!("{}", self.cpu);
            let cpu_info_frag = TextFragment::new(cpu_info)
                .font(self.font)
                .scale(TEXT_SCALE);
            let cpu_info_text = Text::new(cpu_info_frag);
            graphics::draw(
                ctx,
                &cpu_info_text,
                DrawParam::default()
                    .dest([11.0, 9.0])
                    .color(TEXT_BACK_COLOR),
            )?;
            graphics::draw(
                ctx,
                &cpu_info_text,
                DrawParam::default()
                    .dest([10.0, 8.0])
                    .color(TEXT_FRONT_COLOR),
            )?;
        }

        graphics::present(ctx)?;
        timer::yield_now();

        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Escape => ggez::event::quit(ctx),
            KeyCode::Space => self.running = !self.running,
            KeyCode::D => self.show_debug_info = !self.show_debug_info,
            KeyCode::C => {
                if !self.running {
                    if let Err(_) = self.single_clock() {
                        ggez::event::quit(ctx);
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() -> GameResult {
    let window_setup = WindowSetup::default()
        .title(&format!("{} v{}", TITLE, VERSION))
        .vsync(true)
        .srgb(true)
        .samples(NumSamples::One);
    let window_mode = WindowMode::default().dimensions(800.0, 600.0);
    let builder = ContextBuilder::new(TITLE, AUTHOR)
        .window_setup(window_setup)
        .window_mode(window_mode);

    let (mut ctx, event_loop) = builder.build()?;

    const FONT_BYTES: &[u8] = include_bytes!("../res/SourceCodePro-Bold.ttf");
    let font = Font::new_glyph_font_bytes(&mut ctx, FONT_BYTES)?;

    let mut state = EmuState::create(font)?;
    state.reset();

    event::run(ctx, event_loop, state)
}
