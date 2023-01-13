#![feature(new_uninit)]
#![feature(bigint_helper_methods)]

mod cpu;
mod device;

use cpu::Cpu;
use device::{Audio, Controler, ControlerButton, Lcd, Memory, Uart, Vga};

use std::collections::VecDeque;
use std::io::{self, Stdout, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use crossbeam::queue::SegQueue;
use crossterm::{cursor, event, style, terminal};
use crossterm::{ExecutableCommand, QueueableCommand};
use spin_sleep::LoopHelper;

const TITLE: &str = "JAM-1 Emulator";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

const INITIAL_CLOCK_RATE: f64 = 4_000_000.0; // 4 MHz
const FRAME_RATE: f64 = 59.94047619047765; // Actual VGA 60 Hz frequency
const CPU_RESET_PC: u16 = 0xE000;

const UART_BAUD_RATE: f64 = 115_200.0; // 115.2 kHz

const AUDIO_CLOCK_RATE: f64 = 1_843_200.0 / 8.0; // 1.8432 MHz with fixed by 16 divider
const SAMPLE_RATE: u32 = 44100;
const AUDIO_CYCLES_PER_SAMPLE: f64 = AUDIO_CLOCK_RATE / (SAMPLE_RATE as f64);

const VGA_CLOCK_RATE: f64 = 25_175_000.0; // 25.175 MHz
const SCREEN_WIDTH: u16 = 640;
const SCREEN_HEIGHT: u16 = 480;

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
    controler: Controler,

    running: bool,
    fractional_cycles: f64,
    baud_cycles: f64,
    fractional_audio_cycles: f64,
    audio_cycles: f64,
    vga_cycles: f64,
    loop_helper: LoopHelper,
    fps: f64,

    stdout: Stdout,
    input_queue: VecDeque<u8>,
    output_queue: VecDeque<u8>,
    partial_char: Option<Utf8Builder>,
    vga_texture: egui::TextureHandle,
    sample_buffer: Arc<SegQueue<f32>>,

    clock_rate: f64,
    cycles_per_frame: f64,
    whole_cycles_per_frame: u64,
    fract_cycles_per_frame: f64,
    cycles_per_baud: f64,
    audio_cycles_per_cpu_cylce: f64,
    vga_cycles_per_cpu_cycle: f64,
}
impl EmuState {
    pub fn create(
        ui_context: &egui::Context,
        sample_buffer: Arc<SegQueue<f32>>,
    ) -> io::Result<Self> {
        terminal::enable_raw_mode()?;

        let mut stdout = io::stdout();
        stdout.execute(terminal::EnterAlternateScreen)?;
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        stdout.execute(terminal::Clear(terminal::ClearType::Purge))?;
        stdout.execute(cursor::MoveTo(0, 0))?;

        const SCREEN_SIZE: [usize; 2] = [SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize];
        let vga_image = egui::ColorImage::new(SCREEN_SIZE, egui::Color32::BLACK);
        let vga_texture =
            ui_context.load_texture("VGA Framebuffer", vga_image, egui::TextureOptions::NEAREST);

        let clock_rate = INITIAL_CLOCK_RATE;
        let cycles_per_frame = clock_rate / FRAME_RATE;
        let whole_cycles_per_frame = cycles_per_frame as u64;
        let fract_cycles_per_frame = cycles_per_frame - (whole_cycles_per_frame as f64);
        let cycles_per_baud = clock_rate / UART_BAUD_RATE;
        let audio_cycles_per_cpu_cylce = AUDIO_CLOCK_RATE / clock_rate;
        let vga_cycles_per_cpu_cycle = VGA_CLOCK_RATE / clock_rate;

        Ok(Self {
            cpu: Cpu::new(),
            memory: Memory::new(),
            lcd: Lcd::new(),
            uart: Uart::new(),
            audio: Audio::new(),
            vga: Vga::new(),
            controler: Controler::new(),

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
            fps: 0.0,

            stdout,
            input_queue: VecDeque::new(),
            output_queue: VecDeque::new(),
            partial_char: None,
            vga_texture,
            sample_buffer,

            clock_rate,
            cycles_per_frame,
            whole_cycles_per_frame,
            fract_cycles_per_frame,
            cycles_per_baud,
            audio_cycles_per_cpu_cylce,
            vga_cycles_per_cpu_cycle,
        })
    }

    #[inline]
    pub fn reset(&mut self) {
        const MONITOR_BYTES: &[u8] = include_bytes!("../res/Monitor.bin");
        self.memory.init_region(MONITOR_BYTES, CPU_RESET_PC);

        self.cpu.reset(CPU_RESET_PC);
        self.vga.reset();
    }

    pub fn load_program(&mut self, path: &Path) -> io::Result<()> {
        let data = std::fs::read(path)?;
        if data.len() <= 0xE000 {
            self.memory.init_region(&data, 0);
        }

        Ok(())
    }

    fn process_terminal(&mut self) -> io::Result<()> {
        while let Some(data) = self.output_queue.pop_front() {
            if let Some(high_bytes) = &mut self.partial_char {
                if let Some(c) = high_bytes.process_data(data) {
                    self.stdout.queue(style::Print(c))?;
                    self.partial_char = None;
                }
            } else if (data & 0x80) == 0 {
                let c = char::from(data);
                self.stdout.queue(style::Print(c))?;
            } else {
                self.partial_char = Some(Utf8Builder::new(data));
            }
        }

        Ok(())
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

    fn clock(&mut self, n: u64) -> io::Result<()> {
        for _ in 0..n {
            let break_point = self
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
                self.running = false;
                break;
            }
        }

        self.process_terminal()?;
        self.stdout.flush()?;

        Ok(())
    }

    pub fn execute_program(&mut self) -> io::Result<()> {
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

        self.process_terminal()?;
        self.stdout.flush()?;

        self.uart.host_write(b'j');
        self.uart.host_write(b'm');
        self.uart.host_write(b'p');
        self.uart.host_write(b' ');
        self.uart.host_write(b'0');
        self.uart.host_write(b'\r');

        self.running = true;
        Ok(())
    }

    fn clock_frame(&mut self) -> io::Result<()> {
        self.fractional_cycles += self.fract_cycles_per_frame;
        let cycles_to_add = self.fractional_cycles as u64;
        self.fractional_cycles -= cycles_to_add as f64;
        let cycle_count = self.whole_cycles_per_frame + cycles_to_add;

        self.clock(cycle_count)?;

        const SCREEN_SIZE: [usize; 2] = [SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize];
        let vga_image = egui::ColorImage::from_rgba_unmultiplied(
            SCREEN_SIZE,
            self.vga.framebuffer().pixel_data(),
        );
        self.vga_texture
            .set(vga_image, egui::TextureOptions::NEAREST);

        Ok(())
    }

    fn update(&mut self) -> io::Result<()> {
        self.loop_helper.loop_sleep();
        self.loop_helper.loop_start();

        if let Some(fps) = self.loop_helper.report_rate() {
            self.fps = fps;
        }

        while crossterm::event::poll(Duration::ZERO)? {
            let event = crossterm::event::read()?;
            if let crossterm::event::Event::Key(key_event) = event {
                const ESC_SEQ: [u8; 2] = [0x1B, 0x5B];

                match key_event.code {
                    crossterm::event::KeyCode::Backspace => {}
                    crossterm::event::KeyCode::Enter => {
                        self.input_queue.push_back(b'\r');
                        //self.input_queue.push_back(b'\n');
                    }
                    crossterm::event::KeyCode::Left => {
                        self.input_queue.push_back(ESC_SEQ[0]);
                        self.input_queue.push_back(ESC_SEQ[1]);
                        self.input_queue.push_back(b'1');
                        self.input_queue.push_back(b'D');
                    }
                    crossterm::event::KeyCode::Right => {
                        self.input_queue.push_back(ESC_SEQ[0]);
                        self.input_queue.push_back(ESC_SEQ[1]);
                        self.input_queue.push_back(b'1');
                        self.input_queue.push_back(b'C');
                    }
                    crossterm::event::KeyCode::Up => {
                        self.input_queue.push_back(ESC_SEQ[0]);
                        self.input_queue.push_back(ESC_SEQ[1]);
                        self.input_queue.push_back(b'1');
                        self.input_queue.push_back(b'A');
                    }
                    crossterm::event::KeyCode::Down => {
                        self.input_queue.push_back(ESC_SEQ[0]);
                        self.input_queue.push_back(ESC_SEQ[1]);
                        self.input_queue.push_back(b'1');
                        self.input_queue.push_back(b'B');
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

        Ok(())
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        use egui::panel::*;
        use egui::style::*;
        use egui::*;

        SidePanel::new(Side::Right, "ctrl")
            .show_separator_line(false)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.style_mut().wrap = Some(false);

                TopBottomPanel::new(TopBottomSide::Top, "Emulator Control").show_inside(ui, |ui| {
                    if ui
                        .add_enabled(!self.running, Button::new("Load Binary"))
                        .clicked()
                    {
                        let dialog = rfd::FileDialog::new().add_filter("Binary files", &["bin"]);
                        if let Some(path) = dialog.pick_file() {
                            self.load_program(&path).unwrap();
                        }
                    }

                    if self.running {
                        ui.label(format!(
                            "{:.2} fps - {}",
                            self.fps,
                            format_clock_rate(self.fps * self.cycles_per_frame)
                        ));
                    } else {
                        ui.label(format!("{:.2} fps", self.fps));
                    }

                    ui.with_layout(
                        Layout {
                            main_dir: Direction::LeftToRight,
                            ..*ui.layout()
                        },
                        |ui| {
                            if ui
                                .button(if self.running { "Pause" } else { "Run" })
                                .clicked()
                            {
                                self.running = !self.running;
                            }

                            if ui
                                .add_enabled(!self.running, Button::new("Single Step"))
                                .clicked()
                            {
                                self.clock(1).unwrap();
                            }

                            if ui
                                .add_enabled(!self.running, Button::new("Frame Step"))
                                .clicked()
                            {
                                self.clock_frame().unwrap();
                            }

                            if ui.button("Reset").clicked() {
                                self.running = false;
                                self.reset();
                            }
                        },
                    );

                    ui.add_space(10.0);
                });

                TopBottomPanel::new(TopBottomSide::Top, "Register View").show_inside(ui, |ui| {
                    SidePanel::new(Side::Left, "regs16")
                        .show_separator_line(false)
                        .resizable(false)
                        .frame(Frame {
                            inner_margin: Margin::symmetric(35.0, 10.0),
                            ..Default::default()
                        })
                        .show_inside(ui, |ui| {
                            ui.with_layout(ui.layout().with_cross_align(Align::Center), |ui| {
                                ui.label("16 Bit Regs");

                                ui.label(format!("PC: {:0>4X}", self.cpu.pc()));
                                ui.label(format!("RA: {:0>4X}", self.cpu.ra()));
                                ui.label(format!("SP: {:0>4X}", self.cpu.sp()));
                                ui.label(format!("SI: {:0>4X}", self.cpu.si()));
                                ui.label(format!("DI: {:0>4X}", self.cpu.di()));
                                ui.label(format!("TX: {:0>4X}", self.cpu.tx()));
                            });
                        });

                    SidePanel::new(Side::Left, "regs8")
                        .show_separator_line(false)
                        .resizable(false)
                        .frame(Frame {
                            inner_margin: Margin::symmetric(35.0, 10.0),
                            ..Default::default()
                        })
                        .show_inside(ui, |ui| {
                            ui.with_layout(ui.layout().with_cross_align(Align::Center), |ui| {
                                ui.label("8 Bit Regs");

                                ui.label(format!("A:  {:0>2X}", self.cpu.a()));
                                ui.label(format!("B:  {:0>2X}", self.cpu.b()));
                                ui.label(format!("C:  {:0>2X}", self.cpu.c()));
                                ui.label(format!("D:  {:0>2X}", self.cpu.d()));
                                ui.label(format!("TL: {:0>2X}", self.cpu.tl()));
                                ui.label(format!("TH: {:0>2X}", self.cpu.th()));
                            });
                        });

                    SidePanel::new(Side::Right, "flags")
                        .show_separator_line(false)
                        .resizable(false)
                        .frame(Frame {
                            inner_margin: Margin::symmetric(35.0, 10.0),
                            ..Default::default()
                        })
                        .show_inside(ui, |ui| {
                            ui.with_layout(ui.layout().with_cross_align(Align::Center), |ui| {
                                ui.label("Flags");

                                let overflow_val: u8 =
                                    self.cpu.flags().contains(cpu::Flags::OVERFLOW).into();
                                let sign_val: u8 =
                                    self.cpu.flags().contains(cpu::Flags::SIGN).into();
                                let zero_val: u8 =
                                    self.cpu.flags().contains(cpu::Flags::ZERO).into();
                                let carry_a_val: u8 =
                                    self.cpu.flags().contains(cpu::Flags::CARRY_A).into();
                                let carry_l_val: u8 =
                                    self.cpu.flags().contains(cpu::Flags::CARRY_L).into();
                                let flip_val: u8 =
                                    self.cpu.flags().contains(cpu::Flags::PC_RA_FLIP).into();

                                ui.label("F L C Z S O");
                                ui.label(format!(
                                    "{} {} {} {} {} {}",
                                    flip_val,
                                    carry_l_val,
                                    carry_a_val,
                                    zero_val,
                                    sign_val,
                                    overflow_val
                                ));
                            });
                        });
                });

                CentralPanel::default().show_inside(ui, |ui| {
                    ui.with_layout(ui.layout().with_cross_align(Align::Center), |ui| {
                        ui.label("Memory")
                    });

                    ui.label("ADDR | 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F");
                    ui.separator();

                    ScrollArea::new([false, true]).show(ui, |ui| {
                        for addr in (u16::MIN..=u16::MAX).step_by(16) {
                            use std::fmt::Write;

                            let mut text = String::new();
                            write!(text, "{:0>4X} |", addr).unwrap();
                            for i in 0..16 {
                                write!(text, " {:0>2X}", self.memory.read(&self.vga, addr + i))
                                    .unwrap();
                            }
                            ui.label(text);
                        }
                    });
                });
            });

        CentralPanel::default()
            .frame(Frame::side_top_panel(&Style {
                visuals: Visuals {
                    panel_fill: Color32::BLACK,
                    ..Visuals::dark()
                },
                ..Default::default()
            }))
            .show_inside(ui, |ui| {
                const SCREEN_SIZE: Vec2 = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);

                let xf = ui.available_width() / (SCREEN_WIDTH as f32);
                let yf = ui.available_height() / (SCREEN_HEIGHT as f32);
                let f = f32::min(xf, yf);
                let size = SCREEN_SIZE * f;

                ui.centered_and_justified(|ui| {
                    ui.image(self.vga_texture.id(), size);
                })
            });
    }

    fn quit(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        self.stdout.execute(terminal::LeaveAlternateScreen)?;
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?;
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::Purge))?;
        self.stdout.execute(cursor::MoveTo(0, 0))?;
        self.stdout.execute(cursor::Show)?;

        Ok(())
    }
}

//impl EventHandler<GameError> for EmuState {
//
//
//    fn key_down_event(
//        &mut self,
//        ctx: &mut Context,
//        keycode: KeyCode,
//        _keymods: KeyMods,
//        _repeat: bool,
//    ) {
//        match keycode {
//            KeyCode::Escape => ggez::event::quit(ctx),
//            KeyCode::Space => self.running = !self.running,
//            KeyCode::D => self.show_debug_info = !self.show_debug_info,
//            KeyCode::C => {
//                if !self.running {
//                    if let Err(_) = self.clock(1) {
//                        ggez::event::quit(ctx);
//                    }
//                }
//            }
//            KeyCode::F => {
//                if !self.running {
//                    if let Err(_) = self.clock_frame() {
//                        ggez::event::quit(ctx);
//                    }
//                }
//            }
//            KeyCode::R => {
//                self.running = false;
//                self.reset();
//            }
//            KeyCode::O => {
//                if !self.running {
//                    let dialog = rfd::FileDialog::new().add_filter("Binary files", &["bin"]);
//                    if let Some(path) = dialog.pick_file() {
//                        self.load_program(&path);
//                    }
//                }
//            }
//            KeyCode::NumpadAdd => {
//                if self.clock_rate < 64_000_000.0 {
//                    self.clock_rate *= 2.0;
//                    self.cycles_per_frame = self.clock_rate / FRAME_RATE;
//                    self.whole_cycles_per_frame = self.cycles_per_frame as u64;
//                    self.fract_cycles_per_frame =
//                        self.cycles_per_frame - (self.whole_cycles_per_frame as f64);
//                    self.cycles_per_baud = self.clock_rate / UART_BAUD_RATE;
//                    self.audio_cycles_per_cpu_cylce = AUDIO_CLOCK_RATE / self.clock_rate;
//                    self.vga_cycles_per_cpu_cycle = VGA_CLOCK_RATE / self.clock_rate;
//                }
//            }
//            KeyCode::NumpadSubtract => {
//                if self.clock_rate > 1_000.0 {
//                    self.clock_rate /= 2.0;
//                    self.cycles_per_frame = self.clock_rate / FRAME_RATE;
//                    self.whole_cycles_per_frame = self.cycles_per_frame as u64;
//                    self.fract_cycles_per_frame =
//                        self.cycles_per_frame - (self.whole_cycles_per_frame as f64);
//                    self.cycles_per_baud = self.clock_rate / UART_BAUD_RATE;
//                    self.audio_cycles_per_cpu_cylce = AUDIO_CLOCK_RATE / self.clock_rate;
//                    self.vga_cycles_per_cpu_cycle = VGA_CLOCK_RATE / self.clock_rate;
//                }
//            }
//            _ => {}
//        }
//    }
//
//
//}

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

/// Emulator for jam-1 by James Sharman
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Binary file to load and run
    #[clap(short, long, value_parser)]
    run: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use egui_wgpu::winit::Painter;
    use egui_wgpu::WgpuConfiguration;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::EventLoop;
    use winit::window::WindowBuilder;

    let args = Args::parse();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(TITLE)
        .with_inner_size(winit::dpi::PhysicalSize::new(1600, 900))
        .build(&event_loop)?;

    let ui_context = egui::Context::default();
    let mut ui_state = egui_winit::State::new(&event_loop);
    let mut ui_painter = Painter::new(WgpuConfiguration::default(), 1, 0);
    unsafe { ui_painter.set_window(Some(&window)) };

    const FONT: &[u8] = include_bytes!("../res/SourceCodePro-Bold.ttf");
    const FONT_NAME: &str = "SourceCodePro";
    let mut fonts = egui::FontDefinitions::default();
    fonts
        .font_data
        .insert(FONT_NAME.to_owned(), egui::FontData::from_static(FONT));
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, FONT_NAME.to_owned());
    ui_context.set_fonts(fonts);
    ui_context.set_style(egui::Style {
        override_font_id: Some(egui::FontId::monospace(14.0)),
        ..Default::default()
    });

    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let sample_buffer = Arc::new(SegQueue::new());
    let sample_source = SampleSource::new(Arc::clone(&sample_buffer));
    stream_handle.play_raw(sample_source)?;

    let mut gilrs = gilrs::Gilrs::new()?;

    let mut state = EmuState::create(&ui_context, sample_buffer)?;
    state.reset();

    if let Some(path) = &args.run {
        state.load_program(path)?;
        state.execute_program()?;
    }

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                if !ui_state.on_event(&ui_context, &event).consumed {
                    match event {
                        WindowEvent::CloseRequested => {
                            control_flow.set_exit();
                            state.quit().unwrap();
                        }
                        WindowEvent::Resized(size) => {
                            ui_painter.on_window_resized(size.width, size.height)
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
                    match event {
                        gilrs::EventType::ButtonPressed(button, _) => state.button_down(button),
                        gilrs::EventType::ButtonReleased(button, _) => state.button_up(button),
                        _ => {}
                    }
                }

                state.update().unwrap();

                let ui_input = ui_state.take_egui_input(&window);
                let ui_output = ui_context.run(ui_input, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| state.draw(ui));
                });

                ui_state.handle_platform_output(&window, &ui_context, ui_output.platform_output);

                let ui_primitives = ui_context.tessellate(ui_output.shapes);
                ui_painter.paint_and_update_textures(
                    ui_context.pixels_per_point(),
                    egui::Rgba::BLACK,
                    &ui_primitives,
                    &ui_output.textures_delta,
                );
            }
            Event::RedrawEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
