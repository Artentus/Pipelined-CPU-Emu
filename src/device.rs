pub struct Lcd {}
impl Lcd {
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

pub struct Uart {}
impl Uart {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn read_ctrl(&mut self) -> u8 {
        todo!();
    }

    pub fn read_data(&mut self) -> u8 {
        todo!();
    }

    pub fn write_data(&mut self, value: u8) {
        todo!();
    }
}
