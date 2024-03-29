use crate::apu::APU;
use crate::cart::{self, Cart, CartLoadResult};
use crate::ppu::PPU;

#[derive(Debug)]
pub struct System {
    scratch_ram: Box<[u8]>,
    ppu: PPU,
    apu: APU,
    cart: Cart,
}

impl System {
    pub fn new(filename: String) -> CartLoadResult<Self> {
        let cart = cart::load_to_cart(filename)?;

        // TODO: power-on state of `scratch_ram` is funkier than this
        Ok(System {
            scratch_ram: Box::new([0; 0x800]),
            ppu: PPU::new(),
            apu: APU::new(),
            cart,
        })
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        if address < 0x2000 {
            self.scratch_ram[(address & 0x7ff) as usize]
        } else if address < 0x4000 {
            self.ppu.read_address(address)
        } else if address < 0x4020 {
            self.apu.read_address(address)
        } else {
            self.read_mapper_byte(address)
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
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

    pub fn read_word(&self, address: u16) -> u16 {
        let mut output: u16 = 0;
        output += self.read_byte(address + 1) as u16;
        output <<= 8;
        output += self.read_byte(address) as u16;
        output
    }

    fn read_mapper_byte(&self, address: u16) -> u8 {
        if (0x8000..=0xbfff).contains(&address) {
            // We know that `address` is in the first page
            self.cart.prg_rom_pages[0][address as usize - 0x8000]
        } else if address >= 0xc000 {
            self.cart.prg_rom_pages[self.cart.prg_rom_pages.len() - 1][address as usize - 0xc000]
        } else {
            panic!("Cannot read byte at '{}' address from mapper", address);
        }
    }

    fn write_mapper_byte(&self, _address: u16, _value: u8) {}
}
