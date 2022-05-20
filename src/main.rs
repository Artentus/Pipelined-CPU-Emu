#![feature(const_panic)]

mod cpu;
mod device;
#[cfg(test)]
mod tests;
mod types;

use cpu::Cpu;
use device::{Audio, Lcd, Memory, Uart, Vga};
use types::*;

use std::collections::VecDeque;
use std::io::{Stdout, Write};
use std::sync::Arc;
use std::time::Duration;

use crossbeam::queue::SegQueue;
use crossterm::{cursor, style, terminal};
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
const FRAME_RATE: f64 = 59.94047619047765; // Actual VGA 60 Hz frequency
const CYCLES_PER_FRAME: f64 = CLOCK_RATE / FRAME_RATE;
const WHOLE_CYCLES_PER_FRAME: u64 = CYCLES_PER_FRAME as u64;
const FRACT_CYCLES_PER_FRAME: f64 = CYCLES_PER_FRAME - (WHOLE_CYCLES_PER_FRAME as f64);

const UART_BAUD_RATE: f64 = 115_200.0; // 115.2 kHz
const CYCLES_PER_BAUD: f64 = CLOCK_RATE / UART_BAUD_RATE;

const AUDIO_CLOCK_RATE: f64 = 1_843_200.0 / 8.0; // 1.8432 MHz with fixed by 16 divider
const AUDIO_CYCLES_PER_CPU_CYLCE: f64 = AUDIO_CLOCK_RATE / CLOCK_RATE;
const SAMPLE_RATE: u32 = 44100;
const AUDIO_CYCLES_PER_SAMPLE: f64 = AUDIO_CLOCK_RATE / (SAMPLE_RATE as f64);

const VGA_CLOCK_RATE: f64 = 25_175_000.0; // 25.175 MHz
const VGA_CYCLES_PER_CPU_CYCLE: f64 = VGA_CLOCK_RATE / CLOCK_RATE;
const SCREEN_WIDTH: u16 = 640;
const SCREEN_HEIGHT: u16 = 480;
const SCREEN_SCALE: f32 = 2.0;

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

struct SampleSource {
    sample_buffer: Arc<SegQueue<f32>>,
    last_sample: f32,
}
impl SampleSource {
    #[inline]
    pub fn new(sample_buffer: Arc<SegQueue<f32>>) -> Self {
        Self {
            sample_buffer,
            last_sample: 0.0,
        }
    }
}
impl Iterator for SampleSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.sample_buffer.pop() {
            self.last_sample = sample;
            Some(sample)
        } else {
            Some(self.last_sample)
        }
    }
}
impl rodio::Source for SampleSource {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    #[inline]
    fn channels(&self) -> u16 {
        1
    }
    #[inline]
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }
    #[inline]
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

struct EmuState {
    cpu: Cpu,
    memory: Memory,
    lcd: Lcd,
    uart: Uart,
    audio: Audio,
    vga: Vga,

    running: bool,
    fractional_cycles: f64,
    baud_cycles: f64,
    fractional_audio_cycles: f64,
    audio_cycles: f64,
    vga_cycles: f64,
    loop_helper: LoopHelper,

    stdout: Stdout,
    input_queue: VecDeque<u8>,
    output_queue: VecDeque<u8>,
    partial_char: Option<Utf8Builder>,
    sample_buffer: Arc<SegQueue<f32>>,

    font: Font,
    show_debug_info: bool,
}
impl EmuState {
    pub fn create(font: Font, sample_buffer: Arc<SegQueue<f32>>) -> GameResult<Self> {
        //const ROM_BYTES: &[u8] = include_bytes!("../res/snek.bin");
        //const ROM_BYTES: &[u8] = include_bytes!("../res/rom.bin");
        const ROM_BYTES: &[u8] = include_bytes!("../res/Mandlebrot.bin");

        terminal::enable_raw_mode()?;

        let mut stdout = std::io::stdout();
        stdout.execute(terminal::EnterAlternateScreen)?;
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        stdout.execute(terminal::Clear(terminal::ClearType::Purge))?;
        stdout.execute(cursor::MoveTo(0, 0))?;

        Ok(Self {
            cpu: Cpu::new(),
            memory: Memory::from_rom(ROM_BYTES),
            lcd: Lcd::new(),
            uart: Uart::new(),
            audio: Audio::new(),
            vga: Vga::new(),

            running: false,
            fractional_cycles: 0.0,
            baud_cycles: 0.0,
            fractional_audio_cycles: 0.0,
            audio_cycles: 0.0,
            vga_cycles: 0.0,
            loop_helper: LoopHelper::builder()
                .native_accuracy_ns(1_500_000)
                .report_interval_s(0.5)
                .build_with_target_rate(FRAME_RATE),

            stdout,
            input_queue: VecDeque::new(),
            output_queue: VecDeque::new(),
            partial_char: None,
            sample_buffer,

            font,
            show_debug_info: true,
        })
    }

    #[inline]
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.vga.reset();
    }

    fn process_terminal(&mut self) -> GameResult {
        while let Some(data) = self.output_queue.pop_front() {
            if let Some(high_bytes) = &mut self.partial_char {
                if let Some(c) = high_bytes.process_data(data) {
                    self.stdout.queue(style::Print(c))?;
                    self.partial_char = None;
                }
            } else {
                if (data & 0x80) == 0 {
                    let c = char::from(data);
                    self.stdout.queue(style::Print(c))?;
                } else {
                    self.partial_char = Some(Utf8Builder::new(data));
                }
            }
        }

        Ok(())
    }

    fn clock(&mut self, n: u64) -> GameResult {
        for _ in 0..n {
            let break_point = self.cpu.clock(
                &mut self.memory,
                &mut self.lcd,
                &mut self.uart,
                &mut self.audio,
                &mut self.vga,
            );

            self.baud_cycles += 1.0;
            while self.baud_cycles >= CYCLES_PER_BAUD {
                self.baud_cycles -= CYCLES_PER_BAUD;

                if let Some(data) = self.uart.host_read() {
                    self.output_queue.push_back(data);
                }

                if let Some(data) = self.input_queue.pop_front() {
                    self.uart.host_write(data);
                }
            }

            self.fractional_audio_cycles += AUDIO_CYCLES_PER_CPU_CYLCE;
            let whole_audio_cycles = self.fractional_audio_cycles as u32;
            self.fractional_audio_cycles -= whole_audio_cycles as f64;

            for _ in 0..whole_audio_cycles {
                let sample = self.audio.clock();
                self.audio_cycles += 1.0;
                while self.audio_cycles >= AUDIO_CYCLES_PER_SAMPLE {
                    self.audio_cycles -= AUDIO_CYCLES_PER_SAMPLE;

                    self.sample_buffer.push(sample);
                }
            }

            self.vga_cycles += VGA_CYCLES_PER_CPU_CYCLE;
            let whole_vga_cycles = self.vga_cycles as u32;
            self.vga_cycles -= whole_vga_cycles as f64;
            self.vga.clock(&mut self.memory, whole_vga_cycles);
            self.memory.reset_vga_conflict();

            if break_point {
                self.running = false;
                break;
            }
        }

        self.process_terminal()?;
        self.stdout.flush()?;

        Ok(())
    }

    fn clock_frame(&mut self) -> GameResult {
        self.fractional_cycles += FRACT_CYCLES_PER_FRAME;
        let cycles_to_add = self.fractional_cycles as u64;
        self.fractional_cycles -= cycles_to_add as f64;
        let cycle_count = WHOLE_CYCLES_PER_FRAME + cycles_to_add;

        self.clock(cycle_count)
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
                        "{} v{} - {:.2} fps - {}",
                        TITLE,
                        VERSION,
                        fps,
                        format_clock_rate(fps * CYCLES_PER_FRAME)
                    ),
                );
            } else {
                graphics::set_window_title(
                    ctx,
                    &format!("{} v{} - {:.2} fps", TITLE, VERSION, fps),
                );
            }
        }

        while crossterm::event::poll(Duration::ZERO)? {
            let event = crossterm::event::read()?;
            if let crossterm::event::Event::Key(key_event) = event {
                const ESC_SEQ: [u8; 2] = [0x1B, 0x5B];

                match key_event.code {
                    crossterm::event::KeyCode::Backspace => {}
                    crossterm::event::KeyCode::Enter => {}
                    crossterm::event::KeyCode::Left => {
                        self.input_queue.push_back(ESC_SEQ[0]);
                        self.input_queue.push_back(ESC_SEQ[1]);
                        self.input_queue.push_back(31); // ASCII 1
                        self.input_queue.push_back(68); // ASCII D
                    }
                    crossterm::event::KeyCode::Right => {
                        self.input_queue.push_back(ESC_SEQ[0]);
                        self.input_queue.push_back(ESC_SEQ[1]);
                        self.input_queue.push_back(31); // ASCII 1
                        self.input_queue.push_back(67); // ASCII C
                    }
                    crossterm::event::KeyCode::Up => {
                        self.input_queue.push_back(ESC_SEQ[0]);
                        self.input_queue.push_back(ESC_SEQ[1]);
                        self.input_queue.push_back(31); // ASCII 1
                        self.input_queue.push_back(65); // ASCII A
                    }
                    crossterm::event::KeyCode::Down => {
                        self.input_queue.push_back(ESC_SEQ[0]);
                        self.input_queue.push_back(ESC_SEQ[1]);
                        self.input_queue.push_back(31); // ASCII 1
                        self.input_queue.push_back(66); // ASCII B
                    }
                    crossterm::event::KeyCode::Home => {}
                    crossterm::event::KeyCode::End => {}
                    crossterm::event::KeyCode::PageUp => {}
                    crossterm::event::KeyCode::PageDown => {}
                    crossterm::event::KeyCode::Tab => {}
                    crossterm::event::KeyCode::BackTab => {}
                    crossterm::event::KeyCode::Delete => {}
                    crossterm::event::KeyCode::Insert => {}
                    crossterm::event::KeyCode::F(_) => {}
                    crossterm::event::KeyCode::Char(c) => {
                        let mut buffer = [0; 4];
                        let s = c.encode_utf8(&mut buffer);
                        let bytes = s.as_bytes();
                        for b in bytes.iter() {
                            self.input_queue.push_back(*b);
                        }
                    }
                    crossterm::event::KeyCode::Null => {}
                    crossterm::event::KeyCode::Esc => {}
                }
            }
        }

        if self.running {
            self.clock_frame()?;
        }

        timer::yield_now();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);

        let mut screen = Image::from_rgba8(
            ctx,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            self.vga.framebuffer().pixel_data(),
        )?;
        screen.set_filter(FilterMode::Nearest);
        screen.set_wrap(WrapMode::Clamp, WrapMode::Clamp);

        let params = DrawParam::default()
            .dest([0.0, 0.0])
            .scale([SCREEN_SCALE, SCREEN_SCALE]);
        graphics::draw(ctx, &screen, params)?;

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
                    if let Err(_) = self.clock(1) {
                        ggez::event::quit(ctx);
                    }
                }
            }
            KeyCode::F => {
                if !self.running {
                    if let Err(_) = self.clock_frame() {
                        ggez::event::quit(ctx);
                    }
                }
            }
            KeyCode::R => {
                self.running = false;
                self.reset();
            }
            _ => {}
        }
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        let _ = terminal::disable_raw_mode();
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        let _ = self
            .stdout
            .execute(terminal::Clear(terminal::ClearType::All));
        let _ = self
            .stdout
            .execute(terminal::Clear(terminal::ClearType::Purge));
        let _ = self.stdout.execute(cursor::MoveTo(0, 0));
        let _ = self.stdout.execute(cursor::Show);

        false
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window_setup = WindowSetup::default()
        .title(&format!("{} v{}", TITLE, VERSION))
        .vsync(false)
        .srgb(true)
        .samples(NumSamples::One);
    let window_mode = WindowMode::default().dimensions(
        (SCREEN_WIDTH as f32) * SCREEN_SCALE,
        (SCREEN_HEIGHT as f32) * SCREEN_SCALE,
    );
    let builder = ContextBuilder::new(TITLE, AUTHOR)
        .window_setup(window_setup)
        .window_mode(window_mode);

    let (mut ctx, event_loop) = builder.build()?;

    const FONT_BYTES: &[u8] = include_bytes!("../res/SourceCodePro-Bold.ttf");
    let font = Font::new_glyph_font_bytes(&mut ctx, FONT_BYTES)?;

    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let sample_buffer = Arc::new(SegQueue::new());
    let sample_source = SampleSource::new(Arc::clone(&sample_buffer));
    stream_handle.play_raw(sample_source)?;

    let mut state = EmuState::create(font, sample_buffer)?;
    state.reset();

    event::run(ctx, event_loop, state)
}
