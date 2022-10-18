use crate::apu::APU;
use crate::ppu::PPU;

struct System {
    scratch_ram: Box<[u8]>,
    ppu: PPU,
    apu: APU,
}

impl System {
    pub fn new() -> Self {
        // TODO: power-on state of `scratch_ram` is funkier than this
        Self {
            scratch_ram: Box::new([0; 0x800]),
            ppu: PPU::new(),
            apu: APU::new(),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        if address < 0x2000 {
            return self.scratch_ram[(address & 0x7ff) as usize];
        } else if address < 0x4000 {
            return self.ppu.read_address(address);
        } else if address < 0x4020 {
            return self.apu.read_address(address);
        } else {
            return self.read_mapper_byte(address);
        }
    }

    pub fn write_byte(mut self, address: u16, value: u8) {
        if address < 0x2000 {
            self.scratch_ram[(address & 0x7ff) as usize] = value;
        } else if address < 0x4000 {
            self.ppu.write_address(address, value);
        } else if address < 0x4020 {
            self.apu.write_address(address, value);
        } else {
            self.write_mapper_byte(address, value);
        }
    }

    fn read_mapper_byte(&self, address: u16) -> u8 {
        return 0;
    }

    fn write_mapper_byte(&self, address: u16, value: u8) {}
}
