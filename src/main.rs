mod ansi_escaping;
mod syntax_highlighting;

use clap::Parser;
use crossterm::{cursor, style, terminal, ExecutableCommand, QueueableCommand};
use egui_wgpu::winit::Painter;
use jam1emu_lib::*;
use spin_sleep_util::{Interval, RateReporter};
use std::io::{self, Stdout, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use winit::window::Window;

struct NativeTerminal {
    stdout: Stdout,
}

impl vte::Perform for NativeTerminal {
    fn print(&mut self, c: char) {
        use style::*;
        use terminal::*;

        if c == '\x7F' {
            self.stdout.queue(cursor::MoveLeft(1)).unwrap();
            self.stdout.queue(Clear(ClearType::UntilNewLine)).unwrap();
        } else {
            self.stdout.queue(Print(c)).unwrap();
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\r' => {
                self.stdout.queue(cursor::MoveToColumn(0)).unwrap();
            }
            b'\n' => {
                self.stdout.queue(cursor::MoveDown(1)).unwrap();
            }
            b'\x08' => {
                self.stdout.queue(cursor::MoveLeft(1)).unwrap();
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
        use style::*;
        use terminal::*;

        fn take_params<const N: usize>(params: &vte::Params) -> [u16; N] {
            let mut result = [0; N];
            for (i, param) in params.iter().take(N).enumerate() {
                result[i] = param.get(0).copied().unwrap_or(0);
            }
            result
        }

        fn get_color(param: &[u16], params: &mut vte::ParamsIter) -> Option<Color> {
            if param.len() > 1 {
                match param[1] {
                    5 => {
                        let ansi_color = param.get(2).copied().unwrap_or(0);
                        Some(Color::AnsiValue(ansi_color as u8))
                    }
                    2 => {
                        let r = param.get(2).copied().unwrap_or(0) as u8;
                        let g = param.get(3).copied().unwrap_or(0) as u8;
                        let b = param.get(4).copied().unwrap_or(0) as u8;
                        Some(Color::Rgb { r, g, b })
                    }
                    _ => None,
                }
            } else if let Some(&[kind]) = params.next() {
                match kind {
                    5 => {
                        let ansi_color = params.next().map(|p| p[0]).unwrap_or(0);
                        Some(Color::AnsiValue(ansi_color as u8))
                    }
                    2 => {
                        let r = params.next().map(|p| p[0]).unwrap_or(0) as u8;
                        let g = params.next().map(|p| p[0]).unwrap_or(0) as u8;
                        let b = params.next().map(|p| p[0]).unwrap_or(0) as u8;
                        Some(Color::Rgb { r, g, b })
                    }
                    _ => None,
                }
            } else {
                None
            }
        }

        if !ignore {
            match action {
                'A' => {
                    let params = take_params::<1>(params);
                    self.stdout.queue(cursor::MoveUp(params[0].max(1))).unwrap();
                }
                'B' => {
                    let params = take_params::<1>(params);
                    self.stdout
                        .queue(cursor::MoveDown(params[0].max(1)))
                        .unwrap();
                }
                'C' => {
                    let params = take_params::<1>(params);
                    self.stdout
                        .queue(cursor::MoveRight(params[0].max(1)))
                        .unwrap();
                }
                'D' => {
                    let params = take_params::<1>(params);
                    self.stdout
                        .queue(cursor::MoveLeft(params[0].max(1)))
                        .unwrap();
                }
                'E' => {
                    let params = take_params::<1>(params);
                    self.stdout
                        .queue(cursor::MoveToNextLine(params[0].max(1)))
                        .unwrap();
                }
                'F' => {
                    let params = take_params::<1>(params);
                    self.stdout
                        .queue(cursor::MoveToPreviousLine(params[0].max(1)))
                        .unwrap();
                }
                'G' => {
                    let params = take_params::<1>(params);
                    self.stdout.queue(cursor::MoveToColumn(params[0])).unwrap();
                }
                'H' | 'f' => {
                    let params = take_params::<2>(params);
                    self.stdout
                        .queue(cursor::MoveTo(params[1], params[0]))
                        .unwrap();
                }
                'J' => {
                    let params = take_params::<1>(params);
                    match params[0] {
                        0 => {
                            self.stdout.queue(Clear(ClearType::FromCursorDown)).unwrap();
                        }
                        1 => {
                            self.stdout.queue(Clear(ClearType::FromCursorUp)).unwrap();
                        }
                        2 => {
                            self.stdout.queue(Clear(ClearType::All)).unwrap();
                        }
                        3 => {
                            self.stdout.queue(Clear(ClearType::Purge)).unwrap();
                        }
                        _ => {}
                    }
                }
                'K' => {
                    let params = take_params::<1>(params);
                    match params[0] {
                        0 => {
                            self.stdout.queue(Clear(ClearType::UntilNewLine)).unwrap();
                        }
                        1 => {
                            // TODO: erase from start of line up to cursor; not supported by crossterm
                        }
                        2 => {
                            self.stdout.queue(Clear(ClearType::CurrentLine)).unwrap();
                        }
                        _ => {}
                    }
                }
                'h' => {
                    let params = take_params::<1>(params);
                    match params[0] {
                        25 => {
                            self.stdout.queue(cursor::Show).unwrap();
                        }
                        _ => {}
                    }
                }
                'l' => {
                    let params = take_params::<1>(params);
                    match params[0] {
                        25 => {
                            self.stdout.queue(cursor::Hide).unwrap();
                        }
                        _ => {}
                    }
                }
                'm' => {
                    macro_rules! set_attr {
                        ($attr:ident) => {{
                            self.stdout.queue(SetAttribute(Attribute::$attr)).unwrap();
                        }};
                    }

                    macro_rules! set_fg_color {
                        ($color:ident) => {{
                            self.stdout
                                .queue(SetForegroundColor(Color::$color))
                                .unwrap();
                        }};
                    }

                    macro_rules! set_bg_color {
                        ($color:ident) => {{
                            self.stdout
                                .queue(SetBackgroundColor(Color::$color))
                                .unwrap();
                        }};
                    }

                    let mut params = params.iter();
                    while let Some(param) = params.next() {
                        match param[0] {
                            0 => set_attr!(Reset),

                            1 => set_attr!(Bold),
                            2 => set_attr!(Dim),
                            3 => set_attr!(Italic),
                            4 => set_attr!(Underlined),
                            5 => set_attr!(SlowBlink),
                            6 => set_attr!(RapidBlink),
                            7 => set_attr!(Reverse),
                            8 => set_attr!(Hidden),
                            9 => set_attr!(CrossedOut),

                            21 => set_attr!(NormalIntensity),
                            22 => set_attr!(NormalIntensity),
                            23 => set_attr!(NoItalic),
                            24 => set_attr!(NoUnderline),
                            25 => set_attr!(NoBlink),
                            26 => set_attr!(NoBlink),
                            27 => set_attr!(NoReverse),
                            28 => set_attr!(NoHidden),
                            29 => set_attr!(NotCrossedOut),

                            30 => set_fg_color!(Black),
                            31 => set_fg_color!(DarkRed),
                            32 => set_fg_color!(DarkGreen),
                            33 => set_fg_color!(DarkYellow),
                            34 => set_fg_color!(DarkBlue),
                            35 => set_fg_color!(DarkMagenta),
                            36 => set_fg_color!(DarkCyan),
                            37 => set_fg_color!(Grey),
                            38 => {
                                if let Some(color) = get_color(param, &mut params) {
                                    self.stdout.queue(SetForegroundColor(color)).unwrap();
                                }
                            }
                            39 => set_fg_color!(Reset),

                            40 => set_bg_color!(Black),
                            41 => set_bg_color!(DarkRed),
                            42 => set_bg_color!(DarkGreen),
                            43 => set_bg_color!(DarkYellow),
                            44 => set_bg_color!(DarkBlue),
                            45 => set_bg_color!(DarkMagenta),
                            46 => set_bg_color!(DarkCyan),
                            47 => set_bg_color!(Grey),
                            48 => {
                                if let Some(color) = get_color(param, &mut params) {
                                    self.stdout.queue(SetBackgroundColor(color)).unwrap();
                                }
                            }
                            49 => set_bg_color!(Reset),

                            90 => set_fg_color!(DarkGrey),
                            91 => set_fg_color!(Red),
                            92 => set_fg_color!(Green),
                            93 => set_fg_color!(Yellow),
                            94 => set_fg_color!(Blue),
                            95 => set_fg_color!(Magenta),
                            96 => set_fg_color!(Cyan),
                            97 => set_fg_color!(White),

                            100 => set_bg_color!(DarkGrey),
                            101 => set_bg_color!(Red),
                            102 => set_bg_color!(Green),
                            103 => set_bg_color!(Yellow),
                            104 => set_bg_color!(Blue),
                            105 => set_bg_color!(Magenta),
                            106 => set_bg_color!(Cyan),
                            107 => set_bg_color!(White),

                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], ignore: bool, byte: u8) {
        if !ignore {
            match byte {
                b'c' => {
                    use terminal::{Clear, ClearType};

                    self.stdout.queue(Clear(ClearType::All)).unwrap();
                    self.stdout.queue(cursor::MoveTo(0, 0)).unwrap();
                }
                b'3' | b'4' | b'5' | b'6' => {
                    // TODO: double size characters; not supported by crossterm
                }
                _ => {}
            }
        }
    }
}

impl Terminal for NativeTerminal {
    fn reset(&mut self) {
        use terminal::{Clear, ClearType};

        self.stdout.execute(Clear(ClearType::All)).unwrap();
        self.stdout.execute(Clear(ClearType::Purge)).unwrap();
        self.stdout.execute(cursor::Show).unwrap();
        self.stdout.execute(cursor::MoveTo(0, 0)).unwrap();
    }

    #[inline]
    fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }
}

impl NativeTerminal {
    fn new() -> Self {
        terminal::enable_raw_mode().unwrap();

        let mut stdout = io::stdout();
        stdout.execute(terminal::EnterAlternateScreen).unwrap();

        Self { stdout }
    }

    fn quit(&mut self) -> io::Result<()> {
        self.stdout.execute(terminal::LeaveAlternateScreen)?;
        self.stdout.execute(cursor::Show)?;

        terminal::disable_raw_mode()
    }
}

fn process_terminal_input(system: &mut System<NativeTerminal>) {
    use crossterm::event::*;

    const ESC_SEQ: [char; 2] = ['\x1B', '\x5B'];

    while poll(Duration::ZERO).unwrap() {
        let event = read().unwrap();

        if let Event::Key(key_event) = event {
            if matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                match key_event.code {
                    KeyCode::Enter => {
                        system.write_char('\r');
                        //system.write_char('\n');
                    }
                    KeyCode::Backspace => {
                        system.write_char('\x7F');
                    }
                    KeyCode::Left => {
                        system.write_char(ESC_SEQ[0]);
                        system.write_char(ESC_SEQ[1]);
                        system.write_char('1');
                        system.write_char('D');
                    }
                    KeyCode::Right => {
                        system.write_char(ESC_SEQ[0]);
                        system.write_char(ESC_SEQ[1]);
                        system.write_char('1');
                        system.write_char('C');
                    }
                    KeyCode::Up => {
                        system.write_char(ESC_SEQ[0]);
                        system.write_char(ESC_SEQ[1]);
                        system.write_char('1');
                        system.write_char('A');
                    }
                    KeyCode::Down => {
                        system.write_char(ESC_SEQ[0]);
                        system.write_char(ESC_SEQ[1]);
                        system.write_char('1');
                        system.write_char('B');
                    }
                    KeyCode::Char(c) => system.write_char(c),
                    _ => {}
                }
            }
        }
    }
}

struct EmuState {
    running: bool,
    loop_interval: Interval,
    loop_reporter: RateReporter,
    fps: f64,
    vga_texture: egui::TextureHandle,
    code: String,
    assembler_output: String,
    syntax_highlighter: syntax_highlighting::Highlighter,
}

impl EmuState {
    fn create(ui_context: &egui::Context) -> Self {
        let loop_interval = spin_sleep_util::interval(Duration::from_secs_f64(1.0 / FRAME_RATE));
        let loop_reporter = RateReporter::new(Duration::from_secs_f64(0.5));

        let vga_image = egui::ColorImage::new(SCREEN_SIZE, egui::Color32::BLACK);
        let vga_texture =
            ui_context.load_texture("VGA Framebuffer", vga_image, egui::TextureOptions::NEAREST);

        Self {
            running: false,
            loop_interval,
            loop_reporter,
            fps: 0.0,
            vga_texture,
            code: String::new(),
            assembler_output: String::new(),
            syntax_highlighter: Default::default(),
        }
    }

    fn update(&mut self, system: &mut System<NativeTerminal>) {
        self.loop_interval.tick();

        if let Some(fps) = self.loop_reporter.increment_and_report() {
            self.fps = fps;
        }

        process_terminal_input(system);

        if self.running {
            let break_point = system.clock_frame();
            self.running = !break_point;
        }

        let vga_image = egui::ColorImage::from_rgba_unmultiplied(SCREEN_SIZE, system.framebuffer());
        self.vga_texture
            .set(vga_image, egui::TextureOptions::NEAREST);
    }

    fn draw(&mut self, system: &mut System<NativeTerminal>, ui: &mut egui::Ui) {
        use egui::panel::*;
        use egui::style::*;
        use egui::*;

        SidePanel::new(Side::Right, "code")
            .default_width(400.0)
            .show_inside(ui, |ui| {
                TopBottomPanel::new(TopBottomSide::Bottom, "output")
                    .show_separator_line(false)
                    .show_inside(ui, |ui| {
                        if ui
                            .add_enabled(!self.running, Button::new("Assemble"))
                            .clicked()
                        {
                            match assembler::assemble_code(&self.code, false) {
                                Ok((base_addr, data)) => {
                                    if let Err(_) = system.load_program(base_addr, &data) {
                                        self.assembler_output =
                                            "\x1B\x5B1m\x1B\x5B31mError\x1B\x5B39m: assembled binary is too big\x1B\x5B22m".to_owned();
                                    } else {
                                        self.assembler_output = String::new();
                                    }
                                }
                                Err(output) => {
                                    self.assembler_output = output;
                                }
                            }
                        }

                        Frame::dark_canvas(ui.style()).show(ui, |ui| {
                            ScrollArea::both()
                                .min_scrolled_height(200.0)
                                .show(ui, |ui| {
                                    let mut layouter =
                                        |ui: &Ui, string: &str, _: f32| {
                                            ui.fonts(|fonts| {
                                                fonts.layout_job(ansi_escaping::highlight(string))
                                            })
                                        };

                                    TextEdit::multiline(&mut self.assembler_output.as_str())
                                        .desired_width(f32::INFINITY)
                                        .layouter(&mut layouter)
                                        .show(ui);

                                    ui.allocate_space(ui.available_size());
                                });
                        });
                    });

                CentralPanel::default().show_inside(ui, |ui| {
                    ui.allocate_ui(ui.available_size(), |ui| {
                        Frame::dark_canvas(ui.style())
                            .inner_margin(Margin {
                                left: 4.0,
                                right: 4.0,
                                top: 3.0,
                                bottom: 7.0,
                            })
                            .show(ui, |ui| {
                                ScrollArea::vertical().show(ui, |ui| {
                                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                                        // `split` instead of `lines` to preserve the final newline
                                        let code_line_count = self.code.split('\n').count().max(1);

                                        ui.vertical(|ui| {
                                            let max_line_number_size =
                                                code_line_count.ilog10() as usize + 1;
                                            let mut text = String::with_capacity(
                                                code_line_count * (max_line_number_size + 1),
                                            );
                                            for line_number in 1..=code_line_count {
                                                use std::fmt::Write;

                                                if line_number > 1 {
                                                    writeln!(text).unwrap();
                                                }

                                                write!(
                                                    text,
                                                    "{line_number:>width$}",
                                                    width = max_line_number_size,
                                                )
                                                .unwrap();
                                            }

                                            Frame::none().inner_margin(Margin::same(2.0)).show(
                                                ui,
                                                |ui| {
                                                    ui.label(WidgetText::from(text).weak());
                                                },
                                            );
                                        });

                                        Frame::none()
                                            .inner_margin(Margin::symmetric(4.0, 0.0))
                                            .show(ui, |ui| {
                                                ScrollArea::horizontal().show(ui, |ui| {
                                                    let mut layouter =
                                                        |ui: &Ui, string: &str, _: f32| {
                                                            ui.fonts(|fonts| {
                                                                fonts.layout_job(self.syntax_highlighter.highlight(string))
                                                            })
                                                        };

                                                    TextEdit::multiline(&mut self.code)
                                                        .lock_focus(true)
                                                        .desired_width(f32::INFINITY)
                                                        .desired_rows(code_line_count)
                                                        .frame(false)
                                                        .layouter(&mut layouter)
                                                        .show(ui);

                                                    ui.allocate_space(
                                                        ui.available_size() - Vec2::new(8.0, 4.0),
                                                    );
                                                });
                                            });
                                    });

                                    ui.allocate_space(ui.available_size());
                                });
                            });
                    });
                });
            });

        SidePanel::new(Side::Right, "ctrl")
            .show_separator_line(false)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                TopBottomPanel::new(TopBottomSide::Top, "Emulator Control").show_inside(ui, |ui| {
                    if ui
                        .add_enabled(!self.running, Button::new("Load Binary"))
                        .clicked()
                    {
                        let dialog = rfd::FileDialog::new().add_filter("Binary files", &["bin"]);
                        if let Some(program) = dialog.pick_file() {
                            let data = std::fs::read(program).unwrap();
                            system.load_program(0, &data).expect("binary is too big");
                        }
                    }

                    if self.running {
                        ui.label(format!(
                            "{:.2} fps - {}",
                            self.fps,
                            format_clock_rate(self.fps * system.cycles_per_frame())
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
                                system.clock(1);
                            }

                            if ui
                                .add_enabled(!self.running, Button::new("Frame Step"))
                                .clicked()
                            {
                                system.clock_frame();
                            }

                            if ui.button("Reset").clicked() {
                                self.running = false;
                                system.reset();
                            }
                        },
                    );

                    ui.with_layout(
                        Layout {
                            main_dir: Direction::LeftToRight,
                            ..*ui.layout()
                        },
                        |ui| {
                            if ui
                                .add_enabled(
                                    self.running && (system.clock_rate() > 1_000.0),
                                    Button::new("-- Clock Speed"),
                                )
                                .clicked()
                            {
                                system.set_clock_rate(system.clock_rate() * 0.5);
                            }

                            if ui
                                .add_enabled(
                                    self.running && (system.clock_rate() < 16_000_000.0),
                                    Button::new("++ Clock Speed"),
                                )
                                .clicked()
                            {
                                system.set_clock_rate(system.clock_rate() * 2.0);
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

                                ui.label(format!("PC: {:0>4X}", system.cpu().pc()));
                                ui.label(format!("RA: {:0>4X}", system.cpu().ra()));
                                ui.label(format!("SP: {:0>4X}", system.cpu().sp()));
                                ui.label(format!("SI: {:0>4X}", system.cpu().si()));
                                ui.label(format!("DI: {:0>4X}", system.cpu().di()));
                                ui.label(format!("TX: {:0>4X}", system.cpu().tx()));
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

                                ui.label(format!("A:  {:0>2X}", system.cpu().a()));
                                ui.label(format!("B:  {:0>2X}", system.cpu().b()));
                                ui.label(format!("C:  {:0>2X}", system.cpu().c()));
                                ui.label(format!("D:  {:0>2X}", system.cpu().d()));
                                ui.label(format!("TL: {:0>2X}", system.cpu().tl()));
                                ui.label(format!("TH: {:0>2X}", system.cpu().th()));
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
                                    system.cpu().flags().contains(cpu::Flags::OVERFLOW).into();
                                let sign_val: u8 =
                                    system.cpu().flags().contains(cpu::Flags::SIGN).into();
                                let zero_val: u8 =
                                    system.cpu().flags().contains(cpu::Flags::ZERO).into();
                                let carry_a_val: u8 =
                                    system.cpu().flags().contains(cpu::Flags::CARRY_A).into();
                                let carry_l_val: u8 =
                                    system.cpu().flags().contains(cpu::Flags::CARRY_L).into();
                                let flip_val: u8 =
                                    system.cpu().flags().contains(cpu::Flags::PC_RA_FLIP).into();

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

                            // Length of one line is 6 characters for `ADDR |` + 3 characters for each byte.
                            let mut line = String::with_capacity(6 + 16 * 3);
                            write!(line, "{:0>4X} |", addr).unwrap();
                            for i in 0..16 {
                                write!(
                                    line,
                                    " {:0>2X}",
                                    system.memory_view()[(addr as usize) + i],
                                )
                                .unwrap();
                            }

                            ui.label(line);
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

                let xf = ui.available_width() / SCREEN_SIZE.x;
                let yf = ui.available_height() / SCREEN_SIZE.y;
                let f = f32::min(xf, yf);
                let size = SCREEN_SIZE * f;

                ui.centered_and_justified(|ui| {
                    ui.image((self.vga_texture.id(), size));
                })
            });
    }

    #[inline]
    fn quit(&mut self, system: &mut System<NativeTerminal>) {
        system.terminal().quit().unwrap();
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

struct AppState {
    window: Arc<Window>,
    ui_context: egui::Context,
    ui_state: egui_winit::State,
    ui_painter: Painter,
    system: System<NativeTerminal>,
    emu_state: EmuState,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use egui_wgpu::WgpuConfiguration;
    use std::num::NonZeroU32;
    use winit::dpi::PhysicalSize;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::{ControlFlow, EventLoop};
    use winit::window::WindowBuilder;

    let args = Args::parse();
    let event_loop = EventLoop::new()?;
    let mut app_state = None;

    event_loop.run(move |event, window_target| {
        window_target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::Resumed => {
                let window = Arc::new(
                    WindowBuilder::new()
                        .with_title("JAM-1 Emulator")
                        .with_inner_size(PhysicalSize::new(1600, 900))
                        .build(window_target)
                        .unwrap(),
                );

                let ui_context = egui::Context::default();
                let ui_state = egui_winit::State::new(
                    ui_context.clone(),
                    ui_context.viewport_id(),
                    window_target,
                    None,
                    None,
                );
                let gpu_config = WgpuConfiguration::default();
                let mut ui_painter = Painter::new(gpu_config, 1, None, false);

                const FONT: &[u8] = include_bytes!("../res/SourceCodePro-Regular.ttf");
                const FONT_NAME: &str = "SourceCodePro";
                let mut fonts = egui::FontDefinitions::empty();
                fonts
                    .font_data
                    .insert(FONT_NAME.to_owned(), egui::FontData::from_static(FONT));
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, FONT_NAME.to_owned());
                ui_context.set_fonts(fonts);
                ui_context.set_style(egui::Style {
                    override_font_id: Some(egui::FontId::proportional(14.0)),
                    ..Default::default()
                });

                let mut system = System::create(NativeTerminal::new());
                system.reset();

                if let Some(program) = args.run.as_deref() {
                    system
                        .load_program(0, &std::fs::read(program).unwrap())
                        .expect("binary is too big");
                    system.execute_program();
                }

                pollster::block_on(
                    ui_painter.set_window(ui_context.viewport_id(), Some(Arc::clone(&window))),
                )
                .unwrap();

                let emu_state = EmuState::create(&ui_context);

                app_state = Some(AppState {
                    window,
                    ui_context,
                    ui_state,
                    ui_painter,
                    system,
                    emu_state,
                });
            }
            Event::WindowEvent { window_id, event } => {
                if let Some(app_state) = app_state.as_mut() {
                    if window_id == app_state.window.id() {
                        if !app_state
                            .ui_state
                            .on_window_event(&app_state.window, &event)
                            .consumed
                        {
                            match event {
                                WindowEvent::CloseRequested => {
                                    window_target.exit();
                                    app_state.emu_state.quit(&mut app_state.system);
                                }
                                WindowEvent::Resized(size) => {
                                    let width =
                                        NonZeroU32::new(size.width).unwrap_or(NonZeroU32::MIN);
                                    let height =
                                        NonZeroU32::new(size.height).unwrap_or(NonZeroU32::MIN);
                                    app_state.ui_painter.on_window_resized(
                                        app_state.ui_context.viewport_id(),
                                        width,
                                        height,
                                    )
                                }
                                WindowEvent::RedrawRequested => {
                                    app_state.emu_state.update(&mut app_state.system);

                                    let ui_input =
                                        app_state.ui_state.take_egui_input(&app_state.window);
                                    let ui_output = app_state.ui_context.run(ui_input, |ctx| {
                                        egui::CentralPanel::default().show(ctx, |ui| {
                                            app_state.emu_state.draw(&mut app_state.system, ui);
                                        });
                                    });

                                    app_state.ui_state.handle_platform_output(
                                        &app_state.window,
                                        ui_output.platform_output,
                                    );

                                    let ui_primitives = app_state.ui_context.tessellate(
                                        ui_output.shapes,
                                        app_state.ui_context.pixels_per_point(),
                                    );
                                    app_state.ui_painter.paint_and_update_textures(
                                        app_state.ui_context.viewport_id(),
                                        app_state.ui_context.pixels_per_point(),
                                        egui::Rgba::BLACK.to_array(),
                                        &ui_primitives,
                                        &ui_output.textures_delta,
                                        false,
                                    );

                                    app_state.window.request_redraw();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}
