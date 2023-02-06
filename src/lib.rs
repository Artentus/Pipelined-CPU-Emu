#![feature(new_uninit)]
#![feature(bigint_helper_methods)]

pub mod cpu;
mod device;

use cpu::Cpu;
use device::{Audio, Controler, ControlerButton, Lcd, Memory, Uart, Vga};

use crossbeam::queue::SegQueue;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

const INITIAL_CLOCK_RATE: f64 = 4_000_000.0; // 4 MHz
pub const FRAME_RATE: f64 = 59.94047619047765; // Actual VGA 60 Hz frequency
const CPU_RESET_PC: u16 = 0xE000;

const UART_BAUD_RATE: f64 = 115_200.0; // 115.2 kHz

const AUDIO_CLOCK_RATE: f64 = 1_843_200.0 / 8.0; // 1.8432 MHz with fixed by 16 divider
const SAMPLE_RATE: u32 = 44100;
const AUDIO_CYCLES_PER_SAMPLE: f64 = AUDIO_CLOCK_RATE / (SAMPLE_RATE as f64);

const VGA_CLOCK_RATE: f64 = 25_175_000.0; // 25.175 MHz
pub const SCREEN_WIDTH: u16 = 640;
pub const SCREEN_HEIGHT: u16 = 480;
pub const SCREEN_SIZE: [usize; 2] = [SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize];

pub fn format_clock_rate(clock_rate: f64) -> String {
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
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub trait Terminal: vte::Perform {
    fn reset(&mut self);
    fn flush(&mut self);
}

fn map_button(button: gilrs::Button) -> Option<ControlerButton> {
    match button {
        gilrs::Button::South => Some(ControlerButton::B),
        gilrs::Button::East => Some(ControlerButton::A),
        gilrs::Button::North => Some(ControlerButton::X),
        gilrs::Button::West => Some(ControlerButton::Y),
        gilrs::Button::C => None,
        gilrs::Button::Z => None,
        gilrs::Button::LeftTrigger => Some(ControlerButton::L),
        gilrs::Button::LeftTrigger2 => None,
        gilrs::Button::RightTrigger => Some(ControlerButton::R),
        gilrs::Button::RightTrigger2 => None,
        gilrs::Button::Select => Some(ControlerButton::Select),
        gilrs::Button::Start => Some(ControlerButton::Start),
        gilrs::Button::Mode => None,
        gilrs::Button::LeftThumb => None,
        gilrs::Button::RightThumb => None,
        gilrs::Button::DPadUp => Some(ControlerButton::Up),
        gilrs::Button::DPadDown => Some(ControlerButton::Down),
        gilrs::Button::DPadLeft => Some(ControlerButton::Left),
        gilrs::Button::DPadRight => Some(ControlerButton::Right),
        gilrs::Button::Unknown => None,
    }
}

pub struct System<Term: Terminal> {
    cpu: Cpu,
    memory: Memory,
    lcd: Lcd,
    uart: Uart,
    audio: Audio,
    vga: Vga,
    controler: Controler,

    clock_rate: f64,
    cycles_per_frame: f64,
    whole_cycles_per_frame: u64,
    fract_cycles_per_frame: f64,
    cycles_per_baud: f64,
    audio_cycles_per_cpu_cylce: f64,
    vga_cycles_per_cpu_cycle: f64,

    fractional_cycles: f64,
    baud_cycles: f64,
    fractional_audio_cycles: f64,
    audio_cycles: f64,
    vga_cycles: f64,

    input_queue: VecDeque<u8>,
    output_queue: VecDeque<u8>,
    terminal_parser: vte::Parser,
    terminal: Term,
    _audio_stream: rodio::OutputStream,
    sample_buffer: Arc<SegQueue<f32>>,
    gilrs: gilrs::Gilrs,
    memory_view: Vec<u8>,
}

impl<Term: Terminal> System<Term> {
    pub fn create(terminal: Term) -> Result<Self, Box<dyn std::error::Error>> {
        let (_audio_stream, stream_handle) = rodio::OutputStream::try_default()?;
        let sample_buffer = Arc::new(SegQueue::new());
        let sample_source = SampleSource::new(Arc::clone(&sample_buffer));
        stream_handle.play_raw(sample_source)?;

        let mut system = Self {
            cpu: Cpu::new(),
            memory: Memory::new(),
            lcd: Lcd::new(),
            uart: Uart::new(),
            audio: Audio::new(),
            vga: Vga::new(),
            controler: Controler::new(),

            clock_rate: INITIAL_CLOCK_RATE,
            cycles_per_frame: 0.0,
            whole_cycles_per_frame: 0,
            fract_cycles_per_frame: 0.0,
            cycles_per_baud: 0.0,
            audio_cycles_per_cpu_cylce: 0.0,
            vga_cycles_per_cpu_cycle: 0.0,

            fractional_cycles: 0.0,
            baud_cycles: 0.0,
            fractional_audio_cycles: 0.0,
            audio_cycles: 0.0,
            vga_cycles: 0.0,

            input_queue: VecDeque::new(),
            output_queue: VecDeque::new(),
            terminal_parser: vte::Parser::new(),
            terminal,
            _audio_stream,
            sample_buffer,
            gilrs: gilrs::Gilrs::new()?,
            memory_view: Vec::new(),
        };

        system.recalculate_cycles();

        Ok(system)
    }

    fn recalculate_cycles(&mut self) {
        self.cycles_per_frame = self.clock_rate / FRAME_RATE;
        self.whole_cycles_per_frame = self.cycles_per_frame as u64;
        self.fract_cycles_per_frame = self.cycles_per_frame - (self.whole_cycles_per_frame as f64);
        self.cycles_per_baud = self.clock_rate / UART_BAUD_RATE;
        self.audio_cycles_per_cpu_cylce = AUDIO_CLOCK_RATE / self.clock_rate;
        self.vga_cycles_per_cpu_cycle = VGA_CLOCK_RATE / self.clock_rate;
    }

    pub fn reset(&mut self) {
        const MONITOR_BYTES: &[u8] = include_bytes!("../res/Monitor.bin");
        self.memory.init_region(MONITOR_BYTES, CPU_RESET_PC);

        self.cpu.reset(CPU_RESET_PC);
        self.vga.reset();

        self.clock_rate = INITIAL_CLOCK_RATE;
        self.recalculate_cycles();

        self.update_memory_view();
        self.memory_view.shrink_to_fit();

        self.process_terminal();
        self.terminal_parser = vte::Parser::new();
        self.terminal.reset();
    }

    #[inline]
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    #[inline]
    pub fn clock_rate(&self) -> f64 {
        self.clock_rate
    }

    #[inline]
    pub fn set_clock_rate(&mut self, clock_rate: f64) {
        self.clock_rate = clock_rate;
        self.recalculate_cycles();
    }

    #[inline]
    pub fn cycles_per_frame(&self) -> f64 {
        self.cycles_per_frame
    }

    #[inline]
    pub fn framebuffer(&self) -> &[u8] {
        self.vga.framebuffer().pixel_data()
    }

    #[inline]
    pub fn memory_view(&self) -> &[u8] {
        &self.memory_view
    }

    #[inline]
    pub fn terminal(&mut self) -> &mut Term {
        &mut self.terminal
    }

    pub fn write_char(&mut self, c: char) {
        let mut buffer = [0; 4];
        let bytes = c.encode_utf8(&mut buffer).as_bytes();

        for &byte in bytes {
            self.input_queue.push_back(byte);
        }
    }

    fn button_down(&mut self, button: gilrs::Button) {
        if let Some(button) = map_button(button) {
            self.controler.host_button_down(button);
        }
    }

    fn button_up(&mut self, button: gilrs::Button) {
        if let Some(button) = map_button(button) {
            self.controler.host_button_up(button);
        }
    }

    pub fn load_program(&mut self, data: &[u8]) {
        assert!(data.len() <= 0xE000);
        self.memory.init_region(data, 0);
    }

    fn update_memory_view(&mut self) {
        self.memory_view.clear();
        for addr in u16::MIN..=u16::MAX {
            let data = self.memory.read(&self.vga, addr);
            self.memory_view.push(data);
        }
    }

    fn process_terminal(&mut self) {
        while let Some(byte) = self.output_queue.pop_front() {
            self.terminal_parser.advance(&mut self.terminal, byte);
        }
        self.terminal.flush();
    }

    pub fn execute_program(&mut self) {
        let mut loader_finished = false;
        loop {
            self.cpu
                .clock(
                    &mut self.memory,
                    &mut self.lcd,
                    &mut self.uart,
                    &mut self.audio,
                    &mut self.vga,
                    &mut self.controler,
                )
                .expect("invalid instruction");

            self.baud_cycles += 1.0;
            while self.baud_cycles >= self.cycles_per_baud {
                self.baud_cycles -= self.cycles_per_baud;

                if let Some(data) = self.uart.host_read() {
                    self.output_queue.push_back(data);

                    if data == b'>' {
                        loader_finished = true;
                    }
                }
            }

            if loader_finished {
                break;
            }
        }

        self.update_memory_view();
        self.process_terminal();

        self.uart.host_write(b'j');
        self.uart.host_write(b'm');
        self.uart.host_write(b'p');
        self.uart.host_write(b' ');
        self.uart.host_write(b'0');
        self.uart.host_write(b'\r');
    }

    pub fn clock(&mut self, n: u64) -> bool {
        while let Some(gilrs::Event { event, .. }) = self.gilrs.next_event() {
            match event {
                gilrs::EventType::ButtonPressed(button, _) => self.button_down(button),
                gilrs::EventType::ButtonReleased(button, _) => self.button_up(button),
                _ => {}
            }
        }

        let mut break_point = false;
        for _ in 0..n {
            break_point = self
                .cpu
                .clock(
                    &mut self.memory,
                    &mut self.lcd,
                    &mut self.uart,
                    &mut self.audio,
                    &mut self.vga,
                    &mut self.controler,
                )
                .expect("invalid instruction");

            self.baud_cycles += 1.0;
            while self.baud_cycles >= self.cycles_per_baud {
                self.baud_cycles -= self.cycles_per_baud;

                if let Some(data) = self.uart.host_read() {
                    self.output_queue.push_back(data);
                }

                if let Some(data) = self.input_queue.pop_front() {
                    self.uart.host_write(data);
                }
            }

            self.fractional_audio_cycles += self.audio_cycles_per_cpu_cylce;
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

            self.vga_cycles += self.vga_cycles_per_cpu_cycle;
            let whole_vga_cycles = self.vga_cycles as u32;
            self.vga_cycles -= whole_vga_cycles as f64;
            self.vga.clock(&mut self.memory, whole_vga_cycles);
            self.memory.reset_vga_conflict();

            if break_point {
                break;
            }
        }

        self.update_memory_view();
        self.process_terminal();

        break_point
    }

    pub fn clock_frame(&mut self) -> bool {
        self.fractional_cycles += self.fract_cycles_per_frame;
        let cycles_to_add = self.fractional_cycles as u64;
        self.fractional_cycles -= cycles_to_add as f64;
        let cycle_count = self.whole_cycles_per_frame + cycles_to_add;

        self.clock(cycle_count)
    }
}

#[cfg(target_family = "wasm")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::Clamped;

    mod terminal {
        use wasm_bindgen::prelude::*;

        #[wasm_bindgen(module = "/terminal.js")]
        extern "C" {
            pub fn attach();
            pub fn print(s: &str);
            pub fn read_uart_data() -> String;
        }
    }

    struct WebTerminal;

    impl WebTerminal {
        fn new() -> Self {
            terminal::attach();
            Self
        }
    }

    impl vte::Perform for WebTerminal {
        fn print(&mut self, c: char) {
            let mut buffer = [0; 4];
            let s = c.encode_utf8(&mut buffer);
            terminal::print(s);
        }

        fn execute(&mut self, byte: u8) {
            match byte {
                b'\r' => {
                    terminal::print("\x1B\x5B0G");
                }
                b'\n' => {
                    terminal::print("\x1B\x5B1B");
                }
                _ => {}
            }
        }

        fn csi_dispatch(
            &mut self,
            params: &vte::Params,
            _intermediates: &[u8],
            ignore: bool,
            action: char,
        ) {
            if !ignore {
                let mut s = String::new();
                s.push_str("\x1B\x5B");

                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        s.push(';');
                    }

                    for (i, sub_param) in param.iter().enumerate() {
                        if i != 0 {
                            s.push(':');
                        }

                        use std::fmt::Write;
                        write!(s, "{sub_param}").unwrap();
                    }
                }

                s.push(action);
                terminal::print(&s);
            }
        }

        fn esc_dispatch(&mut self, _intermediates: &[u8], ignore: bool, byte: u8) {
            if !ignore && (byte < 0x7F) {
                let bytes = [b'\x1B', byte];
                let s = unsafe { std::str::from_utf8_unchecked(&bytes) };
                terminal::print(s);
            }
        }
    }

    impl super::Terminal for WebTerminal {
        fn reset(&mut self) {
            terminal::print("\x1B\x5B2J");
            terminal::print("\x1B\x5B3J");
            terminal::print("\x1B\x5B25h");
            terminal::print("\x1B\x5B0;0H");
        }

        #[inline]
        fn flush(&mut self) {}
    }

    #[wasm_bindgen]
    pub struct System {
        inner: super::System<WebTerminal>,
    }

    #[wasm_bindgen]
    impl System {
        pub fn create() -> Self {
            Self {
                inner: super::System::create(WebTerminal::new()).unwrap(),
            }
        }

        pub fn reset(&mut self) {
            self.inner.reset();
        }

        pub fn pc(&self) -> u16 {
            self.inner.cpu().pc()
        }

        pub fn ra(&self) -> u16 {
            self.inner.cpu().ra()
        }

        pub fn sp(&self) -> u16 {
            self.inner.cpu().sp()
        }

        pub fn si(&self) -> u16 {
            self.inner.cpu().si()
        }

        pub fn di(&self) -> u16 {
            self.inner.cpu().di()
        }

        pub fn tx(&self) -> u16 {
            self.inner.cpu().tx()
        }

        pub fn a(&self) -> u8 {
            self.inner.cpu().a()
        }

        pub fn b(&self) -> u8 {
            self.inner.cpu().b()
        }

        pub fn c(&self) -> u8 {
            self.inner.cpu().c()
        }

        pub fn d(&self) -> u8 {
            self.inner.cpu().d()
        }

        pub fn tl(&self) -> u8 {
            self.inner.cpu().tl()
        }

        pub fn th(&self) -> u8 {
            self.inner.cpu().th()
        }

        pub fn flags(&self) -> u8 {
            self.inner.cpu().flags().bits()
        }

        pub fn clock_rate(&self) -> f64 {
            self.inner.clock_rate()
        }

        pub fn set_clock_rate(&mut self, clock_rate: f64) {
            self.inner.set_clock_rate(clock_rate);
        }

        pub fn cycles_per_frame(&self) -> f64 {
            self.inner.cycles_per_frame()
        }

        pub fn framebuffer(&self) -> Clamped<Vec<u8>> {
            Clamped(self.inner.framebuffer().to_vec())
        }

        pub fn memory_view(&self) -> Vec<u8> {
            self.inner.memory_view().to_vec()
        }

        pub fn load_program(&mut self, data: &[u8]) {
            self.inner.load_program(data);
        }

        pub fn clock(&mut self, n: u64) -> bool {
            for c in terminal::read_uart_data().chars() {
                self.inner.write_char(c);
            }

            self.inner.clock(n)
        }

        pub fn clock_frame(&mut self) -> bool {
            for c in terminal::read_uart_data().chars() {
                self.inner.write_char(c);
            }

            self.inner.clock_frame()
        }
    }
}
