pub struct Lcd {}
impl Lcd {
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }

    pub fn write_cmd(&mut self, value: u8) {
        todo!();
    }

    pub fn read_data(&mut self) -> u8 {
        todo!();
    }

    pub fn write_data(&mut self, value: u8) {
        todo!();
    }
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

    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == N
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

pub struct Audio {
    shift_data: u32,
    latched_data: u32,
    counter1: u16,
    counter2: u16,
    channel1: bool,
    channel2: bool,
}
impl Audio {
    #[inline]
    pub const fn new() -> Self {
        Self {
            shift_data: 0,
            latched_data: 0xF000F000,
            counter1: 0,
            counter2: 0,
            channel1: false,
            channel2: false,
        }
    }

    #[inline]
    pub fn read_data(&mut self) -> u8 {
        self.latched_data = self.shift_data;

        0
    }

    #[inline]
    pub fn write_data(&mut self, value: u8) {
        self.shift_data = (self.shift_data << 8) | (value as u32);
    }

    pub fn clock(&mut self) -> f32 {
        let freq1 = ((self.latched_data & 0x00000FFF) >> 0) as u16;
        let freq2 = ((self.latched_data & 0x0FFF0000) >> 16) as u16;
        let vol1 = ((self.latched_data & 0x0000F000) >> 12) as u8;
        let vol2 = ((self.latched_data & 0xF0000000) >> 28) as u8;

        if self.counter1 == 0 {
            self.counter1 = u16::max(freq1, 1);
            self.channel1 = !self.channel1;
        }

        if self.counter2 == 0 {
            self.counter2 = u16::max(freq2, 1);
            self.channel2 = !self.channel2;
        }

        self.counter1 -= 1;
        self.counter2 -= 1;

        let norm_vol1 = ((0x0F - vol1) as f32) / 15.0;
        let norm_vol2 = ((0x0F - vol2) as f32) / 15.0;

        let val1 = if self.channel1 { 1.0 } else { 0.0 } * norm_vol1;
        let val2 = if self.channel2 { 1.0 } else { 0.0 } * norm_vol2;

        const MASTER_VOLUME: f32 = 0.05;
        ((val1 + val2) * 2.0 - 1.0) * MASTER_VOLUME
    }
}

pub struct Vga {}
impl Vga {
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    pub fn read_data(&mut self) -> u8 {
        todo!();
    }

    #[inline]
    pub fn write_data(&mut self, value: u8) {
        todo!();
    }
}
