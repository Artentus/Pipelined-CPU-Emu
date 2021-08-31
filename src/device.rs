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
    item_count: usize,
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
            item_count: 0,
        }
    }

    #[inline]
    pub fn item_count(&self) -> usize {
        self.item_count
    }

    pub fn enqueue(&mut self, item: T) -> bool {
        if self.item_count == N {
            // the queue is full
            false
        } else {
            self.items[self.end] = Some(item);

            self.end = (self.end + 1) % N;
            self.item_count += 1;

            true
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.item_count == 0 {
            // the queue is empty
            None
        } else {
            let mut item = None;
            std::mem::swap(&mut self.items[self.start], &mut item);

            self.start = (self.start + 1) % N;
            self.item_count -= 1;

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
        let receive_bytes = self.receive_fifo.item_count() as u8;
        let transmit_bytes = self.transmit_fifo.item_count() as u8;

        receive_bytes | (transmit_bytes << 4)
    }

    #[inline]
    pub fn read_data(&mut self) -> u8 {
        self.receive_fifo.dequeue().unwrap_or(0)
    }

    #[inline]
    pub fn write_data(&mut self, value: u8) {
        let full = self.transmit_fifo.enqueue(value);
        assert!(!full, "Cannot transmit any more data, buffer is full");
    }

    #[inline]
    pub fn host_read(&mut self) -> Option<u8> {
        self.transmit_fifo.dequeue()
    }

    #[inline]
    pub fn host_write(&mut self, value: u8) {
        let full = self.receive_fifo.enqueue(value);
        assert!(!full, "Cannot receive any more data, buffer is full");
    }
}

pub struct Audio {}
impl Audio {
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }

    pub fn read_data(&mut self) -> u8 {
        todo!();
    }

    pub fn write_data(&mut self, value: u8) {
        todo!();
    }
}
