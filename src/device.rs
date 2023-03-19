use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Memory {
    data: Box<[u8]>,
    palette_data: Box<[u8]>,
    framebuffer_conflict: bool,
    last_framebuffer_data: u8,
    palette_conflict: bool,
    last_palette_data: Color,
    palette_high: u8,
    tile_data_conflict: bool,
    last_tile_data: u8,
}
impl Memory {
    const MAP_RANGE_START: u16 = 0x8B00;
    const MAP_RANGE_END: u16 = 0x8C00;
    const VGA_RANGE_START: u16 = 0x8B80;
    const VGA_RANGE_END: u16 = 0x8B84;

    const FRAMEBUFFER_START: u16 = 0xC000;
    const FRAMEBUFFER_END: u16 = 0xE000;
    const FRAMEBUFFER_MASK: u16 = 0x1FFF;

    const PALETTE_START: u16 = 0x8C00;
    const PALETTE_END: u16 = 0x9000;

    const TILE_DATA_START: u16 = 0xA000;
    const TILE_DATA_END: u16 = 0xC000;
    const TILE_DATA_MASK: u16 = 0x1FFF;

    #[inline]
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            data: unsafe { Box::new_zeroed_slice(0x10000).assume_init() },
            palette_data: unsafe { Box::new_zeroed_slice(0x8000).assume_init() },
            framebuffer_conflict: false,
            last_framebuffer_data: 0,
            palette_conflict: false,
            last_palette_data: Color::BLACK,
            palette_high: 0,
            tile_data_conflict: false,
            last_tile_data: 0,
        }
    }

    pub fn init_region(&mut self, data: &[u8], addr: u16) {
        let start = addr as usize;
        let end = start + data.len();
        assert!(end <= 0x10000);

        self.data[start..end].copy_from_slice(data);
    }

    pub fn read(&self, vga: &Vga, addr: u16) -> u8 {
        if (addr >= Self::MAP_RANGE_START) && (addr < Self::MAP_RANGE_END) {
            // Memory mapped IO range

            if (addr >= Self::VGA_RANGE_START) && (addr < Self::VGA_RANGE_END) {
                let vga_addr = addr - Self::VGA_RANGE_START;
                vga.read_mapped_io(vga_addr)
            } else {
                0
            }
        } else {
            self.data[addr as usize]
        }
    }

    pub fn write(&mut self, vga: &mut Vga, addr: u16, value: u8) {
        if (addr >= Self::MAP_RANGE_START) && (addr < Self::MAP_RANGE_END) {
            // Memory mapped IO range

            if (addr >= Self::VGA_RANGE_START) && (addr < Self::VGA_RANGE_END) {
                let vga_addr = addr - Self::VGA_RANGE_START;
                vga.write_mapped_io(vga_addr, value);
            }
        } else {
            // When the CPU writes into the framebuffer or the palette, the VGA is unable
            // to read from that location in order to avoid a bus conflict. If this happens
            // the VGA will effectively read the last valid value.
            if (addr >= Self::FRAMEBUFFER_START) && (addr < Self::FRAMEBUFFER_END) {
                self.framebuffer_conflict = true;
            } else if (addr >= Self::PALETTE_START) && (addr < Self::PALETTE_END) {
                self.palette_conflict = true;

                let palette_addr_high = (self.palette_high as u16) << 10;
                let palette_addr_low = addr & 0x3FF;
                let palette_addr = (palette_addr_high | palette_addr_low) as usize;
                self.palette_data[palette_addr] = value;

                if (addr & 0x3) == 0x3 {
                    self.palette_high = value & 0x1F;
                }
            } else if (addr >= Self::TILE_DATA_START) && (addr < Self::TILE_DATA_END) {
                self.tile_data_conflict = true;
            }

            self.data[addr as usize] = value;
        }
    }

    pub fn framebuffer_read(&mut self, addr: u16) -> u8 {
        // If we currently have a bus conflict we have to return the last value that was read by the VGA.
        if self.framebuffer_conflict {
            self.last_framebuffer_data
        } else {
            let addr = ((addr & Self::FRAMEBUFFER_MASK) + Self::FRAMEBUFFER_START) as usize;
            let data = self.data[addr];
            self.last_framebuffer_data = data;
            data
        }
    }

    pub fn palette_read(&mut self, index: u8) -> Color {
        // If we currently have a bus conflict we have to return the last value that was read by the VGA.
        if self.palette_conflict {
            self.last_palette_data
        } else {
            let palette_addr_high = (self.palette_high as u16) << 10;
            let palette_addr_low = (index as u16) * 4;
            let palette_addr = (palette_addr_high | palette_addr_low) as usize;

            let mut color = Color::BLACK;
            color.channels[0..3]
                .copy_from_slice(&self.palette_data[palette_addr..(palette_addr + 3)]);
            self.last_palette_data = color;
            color
        }
    }

    pub fn tile_data_read(&mut self, addr: u16) -> u8 {
        // If we currently have a bus conflict we have to return the last value that was read by the VGA.
        if self.tile_data_conflict {
            self.last_tile_data
        } else {
            let addr = ((addr & Self::TILE_DATA_MASK) + Self::TILE_DATA_START) as usize;
            let data = self.data[addr];
            self.last_tile_data = data;
            data
        }
    }

    #[inline]
    pub fn reset_vga_conflict(&mut self) {
        self.framebuffer_conflict = false;
        self.palette_conflict = false;
        self.tile_data_conflict = false;
    }
}

pub struct Lcd {}
impl Lcd {
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }

    pub fn write_cmd(&mut self, _value: u8) {}

    pub fn read_cmd(&mut self) -> u8 {
        0
    }

    pub fn write_data(&mut self, _value: u8) {}
}

struct Queue<T, const N: usize> {
    items: [Option<T>; N],
    start: usize,
    end: usize,
    len: usize,
}
impl<T, const N: usize> Queue<T, N> {
    const INIT: Option<T> = None;

    #[inline]
    pub const fn new() -> Self {
        debug_assert!(N > 1);

        Self {
            items: [Self::INIT; N],
            start: 0,
            end: 0,
            len: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn enqueue(&mut self, item: T) -> bool {
        if self.len == N {
            // the queue is full
            false
        } else {
            self.items[self.end] = Some(item);

            self.end = (self.end + 1) % N;
            self.len += 1;

            true
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.len == 0 {
            // the queue is empty
            None
        } else {
            let mut item = None;
            std::mem::swap(&mut self.items[self.start], &mut item);

            self.start = (self.start + 1) % N;
            self.len -= 1;

            Some(item.unwrap())
        }
    }
}

pub struct Uart {
    receive_fifo: Queue<u8, 8>,
    transmit_fifo: Queue<u8, 8>,
}
impl Uart {
    #[inline]
    pub const fn new() -> Self {
        Self {
            receive_fifo: Queue::new(),
            transmit_fifo: Queue::new(),
        }
    }

    // Lower 4 bits count how many received bytes are ready to be read,
    // upper 4 bits count how many bytes have yet to be transmitted
    #[inline]
    pub fn read_ctrl(&mut self) -> u8 {
        let receive_bytes = self.receive_fifo.len() as u8;
        let transmit_bytes = self.transmit_fifo.len() as u8;

        receive_bytes | (transmit_bytes << 4)
    }

    #[inline]
    pub fn read_data(&mut self) -> u8 {
        self.receive_fifo.dequeue().unwrap_or(0)
    }

    #[inline]
    pub fn write_data(&mut self, value: u8) {
        let full = !self.transmit_fifo.enqueue(value);
        assert!(!full, "Cannot transmit any more data, buffer is full");
    }

    #[inline]
    pub fn host_read(&mut self) -> Option<u8> {
        self.transmit_fifo.dequeue()
    }

    #[inline]
    pub fn host_write(&mut self, value: u8) {
        let full = !self.receive_fifo.enqueue(value);
        assert!(!full, "Cannot receive any more data, buffer is full");
    }
}

struct SquareWaveChannel {
    volume: f32,
    frequency: u16,
    counter: u16,
    state: f32,
}
impl SquareWaveChannel {
    #[inline]
    const fn new() -> Self {
        Self {
            volume: 0.0,
            frequency: 0,
            counter: 0,
            state: 1.0,
        }
    }

    fn write(&mut self, data: u16) {
        self.volume = 1.0 - (((data >> 12) as f32) / (0xF as f32));
        self.frequency = data & 0x0FFF;
    }

    fn clock(&mut self) -> f32 {
        if self.counter == 0 {
            self.counter = self.frequency.max(1);
            self.state = -self.state;
        }

        self.counter -= 1;

        self.state * self.volume
    }
}

enum AudioWriteCycleState {
    ChannelSelect,
    LowData,
    HighData,
}

pub struct Audio {
    channel0: SquareWaveChannel,
    channel1: SquareWaveChannel,
    channel2: SquareWaveChannel,
    channel3: SquareWaveChannel,

    cycle_state: AudioWriteCycleState,
    channel_index: u8,
    low_data: u8,
}
impl Audio {
    #[inline]
    pub const fn new() -> Self {
        Self {
            channel0: SquareWaveChannel::new(),
            channel1: SquareWaveChannel::new(),
            channel2: SquareWaveChannel::new(),
            channel3: SquareWaveChannel::new(),

            cycle_state: AudioWriteCycleState::ChannelSelect,
            channel_index: 0,
            low_data: 0,
        }
    }

    #[inline]
    pub fn write_data(&mut self, value: u8) {
        let reset_cycle = (self.channel_index & 0x80) != 0;

        match self.cycle_state {
            AudioWriteCycleState::ChannelSelect => {
                self.channel_index = value;
                self.cycle_state = AudioWriteCycleState::LowData;
            }
            AudioWriteCycleState::LowData => {
                self.low_data = value;
                self.cycle_state = AudioWriteCycleState::HighData;
            }
            AudioWriteCycleState::HighData => {
                let data = u16::from_le_bytes([self.low_data, value]);

                match self.channel_index & 0x7F {
                    0 => self.channel0.write(data),
                    1 => self.channel1.write(data),
                    2 => self.channel2.write(data),
                    3 => self.channel3.write(data),
                    _ => {}
                }

                self.cycle_state = AudioWriteCycleState::ChannelSelect;
            }
        }

        if reset_cycle {
            self.cycle_state = AudioWriteCycleState::ChannelSelect;
        }
    }

    pub fn clock(&mut self) -> f32 {
        let v0 = self.channel0.clock();
        let v1 = self.channel1.clock();
        let v2 = self.channel2.clock();
        let v3 = self.channel3.clock();

        const MASTER_VOLUME: f32 = 0.50;
        (v0 + v1 + v2 + v3).tanh() * MASTER_VOLUME
    }
}

#[repr(C)]
#[repr(align(4))]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Color {
    pub channels: [u8; 4],
}
impl Color {
    pub const BLACK: Color = Color::from_rgb(u8::MIN, u8::MIN, u8::MIN);

    #[inline]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            channels: [r, g, b, u8::MAX],
        }
    }
}

pub struct PixelBuffer {
    pixels: Box<[Color]>,
    width: usize,
    height: usize,
}
impl PixelBuffer {
    #[inline]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![Color::BLACK; width * height].into_boxed_slice(),
            width,
            height,
        }
    }

    #[inline]
    pub fn set_pixel_at(&mut self, col: usize, row: usize, color: Color) {
        debug_assert!(col < self.width);
        debug_assert!(row < self.height);

        self.pixels[col + row * self.width] = color;
    }

    pub fn pixel_data<'a>(&'a self) -> &'a [u8] {
        const COLOR_SIZE: usize = std::mem::size_of::<Color>();

        unsafe {
            let pixel_ptr = self.pixels.as_ptr();
            let data_ptr = pixel_ptr as *const u8;
            let len = self.pixels.len() * COLOR_SIZE;
            std::slice::from_raw_parts(data_ptr, len)
        }
    }
}

pub struct Vga {
    buffer: PixelBuffer,
    h_counter: u16,
    v_counter: u16,
    h_pixel: u16,
    v_pixel: u16,
    h_offset: u16,
    v_offset: u16,
    update_vscroll: bool,
}
impl Vga {
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: PixelBuffer::new(SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize),
            h_counter: 0,
            v_counter: 0,
            h_pixel: 0,
            v_pixel: 0,
            h_offset: 0,
            v_offset: 0,
            update_vscroll: false,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.h_counter = 0;
        self.v_counter = 0;
        self.h_pixel = 0;
        self.v_pixel = 0;
        self.h_offset = 0;
        self.v_offset = 0;
        self.update_vscroll = false;
    }

    #[inline]
    pub fn framebuffer(&self) -> &PixelBuffer {
        &self.buffer
    }

    pub fn read_data(&self) -> u8 {
        const H_SYNC_START: u16 = SCREEN_WIDTH + 16; // Start of horizontal sync (inclusive)
        const H_SYNC_END: u16 = H_SYNC_START + 96; // End of horizontal sync (exclusive)
        const V_SYNC_START: u16 = SCREEN_HEIGHT + 10; // Start of vertical sync (inclusive)
        const V_SYNC_END: u16 = V_SYNC_START + 2; // End of vertical sync (exclusive)
        const LINE_CLOCK_START: u16 = H_SYNC_END + 32; // Start of line clock signal (inclusive)
        const LINE_CLOCK_END: u16 = LINE_CLOCK_START + 16; // End of line clock signal (exclusive)

        const H_SYNC: u8 = 0x80;
        const V_SYNC: u8 = 0x40;
        const H_BLANK: u8 = 0x20;
        const V_BLANK: u8 = 0x10;
        const BLANK: u8 = 0x08;
        const RESET: u8 = 0x04;
        const LINE_CLOCK: u8 = 0x02;

        let mut result = 0;

        if self.h_counter >= SCREEN_WIDTH {
            result |= H_BLANK;
            result |= BLANK;
        }
        if !((self.h_counter >= H_SYNC_START) && (self.h_counter < H_SYNC_END)) {
            // Signal is inverted
            result |= H_SYNC;
        }

        if self.v_counter >= SCREEN_HEIGHT {
            result |= V_BLANK;
            result |= BLANK;
        }
        if !((self.v_counter >= V_SYNC_START) && (self.v_counter < V_SYNC_END)) {
            // Signal is inverted
            result |= V_SYNC;
        }

        if !((self.h_counter == 0) && (self.v_counter == 0)) {
            // Signal is inverted
            result |= RESET;
        }

        if (self.h_counter >= LINE_CLOCK_START) && (self.h_counter < LINE_CLOCK_END) {
            result |= LINE_CLOCK;
        }

        result
    }

    pub fn read_mapped_io(&self, addr: u16) -> u8 {
        match addr {
            0 => self.h_offset.to_le_bytes()[0],
            1 => self.h_offset.to_le_bytes()[1],
            2 => self.v_offset.to_le_bytes()[0],
            3 => self.v_offset.to_le_bytes()[1],
            _ => 0,
        }
    }

    pub fn write_mapped_io(&mut self, addr: u16, value: u8) {
        match addr {
            0 => {
                let mut bytes = self.h_offset.to_le_bytes();
                bytes[0] = value;
                self.h_offset = u16::from_le_bytes(bytes);
            }
            1 => {
                let mut bytes = self.h_offset.to_le_bytes();
                bytes[1] = value;
                self.h_offset = u16::from_le_bytes(bytes);
            }
            2 => {
                let mut bytes = self.v_offset.to_le_bytes();
                bytes[0] = value;
                self.v_offset = u16::from_le_bytes(bytes);
            }
            3 => {
                let mut bytes = self.v_offset.to_le_bytes();
                bytes[1] = value;
                self.v_offset = u16::from_le_bytes(bytes);

                self.update_vscroll = true;
            }
            _ => {}
        }
    }

    pub fn clock(&mut self, mem: &mut Memory, n: u32) {
        const H_PIXELS: u16 = 800; // Number of pixels horizontally (including blanking)
        const V_PIXELS: u16 = 525; // Number of pixels vertically (including blanking)

        // In hardware the scroll offsets include the front porch region of the screen.
        const BASE_H_OFFSET: u16 = 47;
        const BASE_V_OFFSET: u16 = 33;

        for _ in 0..n {
            self.h_counter += 1;
            self.h_pixel = self.h_pixel.wrapping_add(1);

            if self.h_counter == H_PIXELS {
                self.h_counter = 0;
                self.h_pixel = self.h_offset.wrapping_add(BASE_H_OFFSET);

                self.v_counter += 1;
                if self.update_vscroll {
                    self.v_pixel = self.v_offset.into();
                } else {
                    self.v_pixel = self.v_pixel.wrapping_add(1);
                }

                if self.v_counter == V_PIXELS {
                    self.v_counter = 0;
                    self.v_pixel = self.v_offset.wrapping_add(BASE_V_OFFSET);
                    self.update_vscroll = false;
                }
            }

            if (self.h_counter < SCREEN_WIDTH) && (self.v_counter < SCREEN_HEIGHT) {
                let h_addr = (self.h_pixel >> 3) & 0x7F;
                let v_addr = (self.v_pixel >> 3) & 0x3F;
                let tile_x = self.h_pixel & 0x7;
                let tile_y = self.v_pixel & 0x7;

                let framebuffer_addr = h_addr | (v_addr << 7);
                let tile_index = mem.framebuffer_read(framebuffer_addr);

                let tile_addr = ((tile_index as u16) << 5) | (tile_y << 2) | (tile_x >> 1);
                let nibble_shift = (tile_x & 0x1) * 4;
                let palette_index = (mem.tile_data_read(tile_addr) >> nibble_shift) & 0xF;

                let color = mem.palette_read(palette_index);

                self.buffer
                    .set_pixel_at(self.h_counter as usize, self.v_counter as usize, color);
            }
        }
    }
}

// Follows the SNES layout
pub enum ControlerButton {
    A,
    B,
    X,
    Y,
    Up,
    Down,
    Left,
    Right,
    R,
    L,
    Start,
    Select,
}

pub struct Controler {
    low: u8,
    high: u8,
    state: bool,
}
impl Controler {
    pub fn new() -> Self {
        Self {
            low: 0,
            high: 0,
            state: false,
        }
    }

    pub fn host_button_down(&mut self, button: ControlerButton) {
        match button {
            ControlerButton::A => self.high |= 0x1,
            ControlerButton::B => self.low |= 0x01,
            ControlerButton::X => self.high |= 0x2,
            ControlerButton::Y => self.low |= 0x02,
            ControlerButton::Up => self.low |= 0x10,
            ControlerButton::Down => self.low |= 0x20,
            ControlerButton::Left => self.low |= 0x40,
            ControlerButton::Right => self.low |= 0x80,
            ControlerButton::R => self.high |= 0x8,
            ControlerButton::L => self.high |= 0x4,
            ControlerButton::Start => self.low |= 0x08,
            ControlerButton::Select => self.low |= 0x04,
        }
    }

    pub fn host_button_up(&mut self, button: ControlerButton) {
        match button {
            ControlerButton::A => self.high &= !0x1,
            ControlerButton::B => self.low &= !0x01,
            ControlerButton::X => self.high &= !0x2,
            ControlerButton::Y => self.low &= !0x02,
            ControlerButton::Up => self.low &= !0x10,
            ControlerButton::Down => self.low &= !0x20,
            ControlerButton::Left => self.low &= !0x40,
            ControlerButton::Right => self.low &= !0x80,
            ControlerButton::R => self.high &= !0x8,
            ControlerButton::L => self.high &= !0x4,
            ControlerButton::Start => self.low &= !0x08,
            ControlerButton::Select => self.low &= !0x04,
        }
    }

    pub fn read_data(&mut self) -> u8 {
        let result = if self.state {
            self.high | 0x80 // The msb of the high byte is pulled high to identify controller state
        } else {
            self.low
        };

        self.state = !self.state;
        result
    }
}
