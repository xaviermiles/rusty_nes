#[derive(Debug)]
pub struct APU {}

impl APU {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_address(&self, _address: u16) -> u8 {
        return 0;
    }

    pub fn write_address(&self, _address: u16, _value: u8) {}
}
