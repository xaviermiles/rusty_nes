#[derive(Debug)]
pub struct PPU {}

impl PPU {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_address(&self, address: u16) -> u8 {
        return 0;
    }

    pub fn write_address(&self, address: u16, value: u8) {}
}
