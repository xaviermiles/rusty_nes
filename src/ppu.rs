/// Picture Processing Unit (PPU)
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub struct PPU {
    in_vblank: bool,
}

impl PPU {
    pub fn new() -> Self {
        Self { in_vblank: false }
    }

    pub fn read_address(&mut self, _address: u16) -> u8 {
        let mut status = 0;
        if self.in_vblank {
            status |= 0x80;
        }
        self.in_vblank = false;
        status
    }

    pub fn write_address(&self, _address: u16, _value: u8) {}
}
