#![feature(maybe_uninit_extra)]
#![feature(const_panic)]

mod cpu;
mod device;
mod types;

use cpu::Cpu;
use device::{Lcd, Uart};
use types::*;

use std::collections::VecDeque;
use std::io::{Stdout, Write};

use crossterm::cursor::{MoveDown, MoveToColumn};
use crossterm::style::Print;
use crossterm::QueueableCommand;
use ggez::conf::{NumSamples, WindowMode, WindowSetup};
use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics::{
    Color, DrawParam, FilterMode, Font, Image, PxScale, Text, TextFragment, WrapMode,
};
use ggez::{event, graphics, timer, Context, ContextBuilder, GameError, GameResult};
use spin_sleep::LoopHelper;

const TITLE: &str = "rEmu";
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

    #[inline]
    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    #[inline]
    pub fn write(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value;
    }
}

struct EmuState {
    cpu: Cpu,
    memory: Box<Memory>,
    lcd: Lcd,
    uart: Uart,

    running: bool,
    fractional_cycles: f64,
    baud_cycles: f64,
    loop_helper: LoopHelper,

    stdout: Stdout,
    input_queue: VecDeque<u8>,

    font: Font,
    show_debug_info: bool,
}
impl EmuState {
    pub fn create(font: Font) -> Self {
        // ToDo: load ROM

        Self {
            cpu: Cpu::new(),
            memory: Memory::create(),
            lcd: Lcd::new(),
            uart: Uart::new(),

            running: false,
            fractional_cycles: 0.0,
            baud_cycles: 0.0,
            loop_helper: LoopHelper::builder()
                .native_accuracy_ns(1_500_000)
                .report_interval_s(0.5)
                .build_with_target_rate(FRAME_RATE),

            stdout: std::io::stdout(),
            input_queue: VecDeque::new(),

            font,
            show_debug_info: true,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    fn process_uart(&mut self) -> GameResult {
        if let Some(data) = self.uart.host_read() {
            if data == 0x09 {
                self.stdout.queue(Print('\t'))?;
            } else if data == 0x0A {
                self.stdout.queue(MoveDown(1))?;
            } else if data == 0x0D {
                self.stdout.queue(MoveToColumn(0))?;
            } else {
                let c = char::from(data);

                if c.is_ascii() && !c.is_ascii_control() {
                    self.stdout.queue(Print(c))?;
                } else {
                    self.stdout.queue(Print(char::REPLACEMENT_CHARACTER))?;
                }
            }
        }

        if let Some(data) = self.input_queue.pop_front() {
            self.uart.host_write(data);
        }

        Ok(())
    }

    fn clock(&mut self) -> GameResult {
        self.cpu
            .clock(self.memory.as_mut(), &mut self.lcd, &mut self.uart);

        self.baud_cycles += 1.0;
        while self.baud_cycles >= CYCLES_PER_BAUD {
            self.baud_cycles -= CYCLES_PER_BAUD;

            self.process_uart()?;
        }

        Ok(())
    }

    fn single_clock(&mut self) -> GameResult {
        self.clock()?;
        self.stdout.flush()?;

        Ok(())
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
                    self.clock()?;
                }
            }
        }

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

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        // ToDo
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

    let mut state = EmuState::create(font);
    state.reset();

    event::run(ctx, event_loop, state)
}
