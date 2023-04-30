const CLOCKS_PER_FRAME: u64 = 341 * 262;
const CLOCKS_PER_FRAME_BEFORE_VBLANK: u64 = 341 * 242;

/// Picture Processing Unit (PPU)
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub struct PPU {
    clock: u64,
    in_vblank: bool,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            clock: 0,
            in_vblank: false,
        }
    }

    /// Tick the PPU's clock by a number of `cycles`.
    pub fn tick(&mut self, cycles: u64) {
        self.clock += cycles;

        let clock_in_current_frame = self.clock % CLOCKS_PER_FRAME;
        if !self.in_vblank && clock_in_current_frame > CLOCKS_PER_FRAME_BEFORE_VBLANK {
            self.in_vblank = true;
        } else if self.in_vblank && clock_in_current_frame <= CLOCKS_PER_FRAME_BEFORE_VBLANK {
            self.in_vblank = false;
        }
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
