mod cpu;
mod types;

use cpu::Cpu;
use types::*;

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

fn main() {
    let mut cpu = Cpu::new();
    let mut mem = Memory::create();

    cpu.reset();
    cpu.clock(mem.as_mut());
}
