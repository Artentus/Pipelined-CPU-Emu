mod cpu;
mod device;
mod types;

use cpu::Cpu;
use device::{Lcd, Uart};
use types::*;

use ggez::conf::{NumSamples, WindowMode, WindowSetup};
use ggez::event::{EventHandler, KeyCode};
use spin_sleep::LoopHelper;
//use ggez::graphics::{DrawParam, FilterMode, Font, Image, WrapMode};
use ggez::{event, graphics, timer, Context, ContextBuilder, GameResult};

const TITLE: &str = "rEmu";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

const CLOCK_RATE: f64 = 1_000_000.0; // 1 MHz
const FRAME_RATE: u32 = 60;
const CYCLES_PER_FRAME: f64 = CLOCK_RATE / (FRAME_RATE as f64);
const WHOLE_CYCLES_PER_FRAME: u64 = CYCLES_PER_FRAME as u64;
const FRACT_CYCLES_PER_FRAME: f64 = CYCLES_PER_FRAME - (WHOLE_CYCLES_PER_FRAME as f64);

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

    fractional_cycles: f64,
    loop_helper: LoopHelper,
}
impl EmuState {
    pub fn create() -> Self {
        // ToDo: load ROM

        Self {
            cpu: Cpu::new(),
            memory: Memory::create(),
            lcd: Lcd::new(),
            uart: Uart::new(),

            fractional_cycles: 0.0,
            loop_helper: LoopHelper::builder()
                .native_accuracy_ns(1_500_000)
                .report_interval_s(0.5)
                .build_with_target_rate(FRAME_RATE),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    #[inline]
    pub fn clock(&mut self) {
        self.cpu
            .clock(self.memory.as_mut(), &mut self.lcd, &mut self.uart);
    }
}
impl EventHandler for EmuState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.loop_helper.loop_start();
        if let Some(fps) = self.loop_helper.report_rate() {
            graphics::set_window_title(
                ctx,
                &format!(
                    "{} v{} - {:.1} fps - {:.1} MHz",
                    TITLE,
                    VERSION,
                    fps,
                    fps * CYCLES_PER_FRAME / 1_000_000.0
                ),
            );
        }

        while timer::check_update_time(ctx, FRAME_RATE) {
            self.fractional_cycles += FRACT_CYCLES_PER_FRAME;
            let cycles_to_add = self.fractional_cycles as u64;
            self.fractional_cycles -= cycles_to_add as f64;
            let cycle_count = WHOLE_CYCLES_PER_FRAME + cycles_to_add;

            for _ in 0..cycle_count {
                self.clock();
            }
        }

        timer::yield_now();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, ggez::graphics::BLACK);

        // ToDo: VGA

        graphics::present(ctx)?;
        timer::yield_now();

        self.loop_helper.loop_sleep();

        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: event::KeyCode,
        _keymods: event::KeyMods,
        _repeat: bool,
    ) {
        // ToDo
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: event::KeyMods) {
        // ToDo
    }
}

fn main() -> GameResult {
    let mut state = EmuState::create();
    state.reset();

    let window_setup = WindowSetup::default()
        .title(&format!("{} v{}", TITLE, VERSION))
        .vsync(true)
        .srgb(true)
        .samples(NumSamples::Zero);
    let window_mode = WindowMode::default().dimensions(800.0, 600.0);
    let builder = ContextBuilder::new(TITLE, AUTHOR)
        .window_setup(window_setup)
        .window_mode(window_mode);

    let (ref mut ctx, ref mut event_loop) = builder.build()?;
    event::run(ctx, event_loop, &mut state)?;

    Ok(())
}
