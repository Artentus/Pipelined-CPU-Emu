use clap::Parser;
use crossterm::{cursor, style, terminal, ExecutableCommand, QueueableCommand};
use jam1emu_lib::*;
use spin_sleep::LoopHelper;
use std::io::{self, Stdout, Write};
use std::path::PathBuf;
use std::time::Duration;

struct NativeTerminal {
    stdout: Stdout,
}

impl vte::Perform for NativeTerminal {
    #[inline]
    fn print(&mut self, c: char) {
        self.stdout.queue(style::Print(c)).unwrap();
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\r' => {
                self.stdout.queue(cursor::MoveToColumn(0)).unwrap();
            }
            b'\n' => {
                self.stdout.queue(cursor::MoveDown(1)).unwrap();
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
                            49 => set_fg_color!(Reset),

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
    loop_helper: LoopHelper,
    fps: f64,
    vga_texture: egui::TextureHandle,
}

impl EmuState {
    fn create(ui_context: &egui::Context) -> Self {
        const NATIVE_TIMER_ACCURACY: u32 = 1_500_000;
        let loop_helper = LoopHelper::builder()
            .native_accuracy_ns(NATIVE_TIMER_ACCURACY)
            .report_interval_s(0.5)
            .build_with_target_rate(FRAME_RATE);

        let vga_image = egui::ColorImage::new(SCREEN_SIZE, egui::Color32::BLACK);
        let vga_texture =
            ui_context.load_texture("VGA Framebuffer", vga_image, egui::TextureOptions::NEAREST);

        Self {
            running: false,
            loop_helper,
            fps: 0.0,
            vga_texture,
        }
    }

    fn update(&mut self, system: &mut System<NativeTerminal>) {
        self.loop_helper.loop_sleep();
        self.loop_helper.loop_start();

        if let Some(fps) = self.loop_helper.report_rate() {
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
                        if let Some(program) = dialog.pick_file() {
                            let data = std::fs::read(program).unwrap();
                            system.load_program(&data);
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
                    ui.image(self.vga_texture.id(), size);
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use egui_wgpu::winit::Painter;
    use egui_wgpu::WgpuConfiguration;
    use winit::dpi::PhysicalSize;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::EventLoop;
    use winit::window::WindowBuilder;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("JAM-1 Emulator")
        .with_inner_size(PhysicalSize::new(1600, 900))
        .build(&event_loop)?;

    let ui_context = egui::Context::default();
    let mut ui_state = egui_winit::State::new(&event_loop);
    let mut gpu_config = WgpuConfiguration::default();
    gpu_config.device_descriptor.limits = wgpu::Limits::downlevel_webgl2_defaults();
    let mut ui_painter = Painter::new(gpu_config, 1, 0);
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

    let mut system = System::create(NativeTerminal::new())?;
    system.reset();
    let mut state = EmuState::create(&ui_context);

    let args = Args::parse();
    if let Some(program) = args.run {
        system.load_program(&std::fs::read(program)?);
        system.execute_program();
    }

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                if !ui_state.on_event(&ui_context, &event).consumed {
                    match event {
                        WindowEvent::CloseRequested => {
                            control_flow.set_exit();
                            state.quit(&mut system);
                        }
                        WindowEvent::Resized(size) => {
                            ui_painter.on_window_resized(size.width, size.height)
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update(&mut system);

                let ui_input = ui_state.take_egui_input(&window);
                let ui_output = ui_context.run(ui_input, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| state.draw(&mut system, ui));
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
