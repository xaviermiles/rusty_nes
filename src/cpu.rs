use crate::system::System;

/// The 2A03 NES CPU core, which is based on the 6502 processor
///
/// See: https://www.nesdev.org/wiki/CPU_registers
pub struct CPU<'a> {
    /// Accumulator
    a: u8,

    /// Indexes used for several addressing modes
    x: u8,
    y: u8,

    /// Program counter
    pc: u16,

    /// Stack pointer
    s: u8,

    /// Status register flags
    ///
    /// See: https://www.nesdev.org/wiki/Status_flags
    carry: bool,
    zero: bool,
    interrupt_disable: bool,
    decimal: bool,
    // TODO: add No CPU Effect (the B flag)
    overflow: bool,
    negative: bool,

    /// System
    system: &'a mut System,

    /// Clock
    clock: u64,
}

impl<'a> CPU<'a> {
    /// Create a new CPU, in the power up state
    ///
    /// See: https://www.nesdev.org/wiki/CPU_power_up_state
    pub fn new(system: &'a System) -> Self {
        let reset_vector = (&system.read_word(0xfffc)).clone();

        Self {
            a: 0,
            x: 0,
            y: 0,
            s: 0xfd,
            pc: reset_vector,
            carry: false,
            zero: false,
            interrupt_disable: true,
            decimal: false,
            overflow: false,
            negative: false,
            system,
            clock: 0,
        }
    }

    pub fn pretty_print(&self) {
        print!(
            "a: {:02x} x: {:02x} y: {:02x} pc: {:04x} s: {:02x} flags: ",
            self.a, self.x, self.y, self.pc, self.s
        );
        print!("{}", if self.negative { "N" } else { "-" });
        print!("{}", if self.overflow { "V" } else { "-" });
        print!("{}", if self.decimal { "D" } else { "-" });
        print!("{}", if self.interrupt_disable { "I" } else { "-" });
        print!("{}", if self.zero { "Z" } else { "-" });
        print!("{}", if self.carry { "C" } else { "-" });
    }

    /// LoaD Accumulator
    fn lda(&mut self, opcode: u8) {
        let arg_address = self.pc + 1;

        let intermediate = match opcode {
            0xa9 => {
                // Immediate (imm)
                self.clock += 2;
                self.pc += 2;

                self.system.read_byte(arg_address)
            }
            0xa5 => {
                // Zero page (zp)
                self.clock += 3;
                self.pc += 2;

                let address = self.system.read_byte(arg_address) as u16;
                self.system.read_byte(address)
            }
            0xb5 => {
                // Zero page, x (zpx)
                self.clock += 4;
                self.pc += 2;

                let address = (self.system.read_byte(arg_address) + self.x) as u16;
                self.system.read_byte(address)
            }
            0xad => {
                // Absolute address (abs)
                self.clock += 4;
                self.pc += 3;

                let address = self.system.read_word(arg_address);
                self.system.read_byte(address)
            }
            0xbd => {
                // Absolute address, x (abx)
                self.clock += 6;
                self.pc += 3;

                let mut address = self.system.read_word(arg_address);
                let page1 = address >> 2;

                address += self.x as u16;
                let page2 = address >> 2;
                if page1 != page2 {
                    self.clock += 1;
                }

                self.system.read_byte(address)
            }
            0xb9 => {
                // Absolute address, y (aby)
                self.clock += 4;
                self.pc += 2;

                let mut address = self.system.read_word(arg_address);
                let page = address >> 2;

                address += self.y as u16;
                let new_page = address >> 2;
                if page != new_page {
                    self.clock += 1;
                }

                self.system.read_byte(address)
            }
            0xa1 => {
                // Indirect zero page, x (izx)
                self.clock += 6;
                self.pc += 4;

                let address = (self.system.read_byte(arg_address) + self.x) as u16;
                let indirect_address = self.system.read_word(address);
                self.system.read_byte(indirect_address)
            }
            0xb1 => {
                // Indirect zero page, y (izy)
                self.clock += 6;
                self.pc += 2;

                let address = (self.system.read_byte(arg_address) + self.x) as u16;

                let pre_index = self.system.read_word(address);
                let page1 = pre_index >> 2;
                let indirect_address = pre_index + self.y as u16;
                let page2 = indirect_address >> 2;
                if page1 != page2 {
                    self.clock += 1;
                }

                self.system.read_byte(indirect_address)
            }
            _ => {
                panic!("Unknown opcode");
            }
        };

        // Set the flags
        self.negative = intermediate & 0x80 == 0x80;
        self.zero = intermediate == 0;

        self.a = intermediate;
    }

    /// LoaD X register
    fn ldx(&mut self, opcode: u8) {
        let arg_address = self.pc + 1;

        let intermediate = match opcode {
            0xa2 => {
                // Immediate
                self.clock += 2;
                self.pc += 2;

                self.system.read_byte(arg_address)
            }
            0xa6 => {
                // Zero page (zp)
                self.clock += 3;
                self.pc += 2;

                let address = self.system.read_byte(arg_address) as u16;
                self.system.read_byte(address)
            }
            0xb6 => {
                // Zero page, y (zpy)
                self.clock += 4;
                self.pc += 2;

                let address = (self.system.read_byte(arg_address) + self.y) as u16;
                self.system.read_byte(address)
            }
            0xae => {
                // Absolute address (abs)
                self.clock += 4;
                self.pc += 3;

                let address = self.system.read_word(arg_address);
                self.system.read_byte(address)
            }
            0xbe => {
                // Absolute address, y (aby)
                self.clock += 4;
                self.pc += 2;

                let mut address = self.system.read_word(arg_address);
                let page = address >> 2;

                address += self.y as u16;
                let new_page = address >> 2;
                if page != new_page {
                    self.clock += 1;
                }

                self.system.read_byte(address)
            }
            _ => {
                panic!("Unknown opcode");
            }
        };

        self.negative = intermediate & 0x80 == 0x80;
        self.zero = intermediate == 0;

        self.x = intermediate;
    }

    /// LoaD Y register
    fn ldy(&mut self, opcode: u8) {
        let arg_address = self.pc + 1;

        let intermediate = match opcode {
            0xa0 => {
                // Immediate
                self.clock += 2;
                self.pc += 2;

                self.system.read_byte(arg_address)
            }
            0xa4 => {
                // Zero page (zp)
                self.clock += 3;
                self.pc += 2;

                let address = self.system.read_byte(arg_address) as u16;
                self.system.read_byte(address)
            }
            0xb4 => {
                // Zero page, x (zpx)
                self.clock += 4;
                self.pc += 2;

                let address = (self.system.read_byte(arg_address) + self.x) as u16;
                self.system.read_byte(address)
            }
            0x8c => {
                // Absolute address (abs)
                self.clock += 4;
                self.pc += 3;

                let address = self.system.read_word(arg_address);
                self.system.read_byte(address)
            }
            0xbc => {
                // Absolute address, x (abx)
                self.clock += 4;
                self.pc += 2;

                let mut address = self.system.read_word(arg_address);
                let page = address >> 2;

                address += self.x as u16;
                let new_page = address >> 2;
                if page != new_page {
                    self.clock += 1;
                }

                self.system.read_byte(address)
            }
            _ => {
                panic!("Unknown opcode");
            }
        };

        self.negative = intermediate & 0x80 == 0x80;
        self.zero = intermediate == 0;

        self.y = intermediate;
    }

    /// STore Accumulator
    fn sta(&mut self, opcode: u8) {
        let arg_address = self.pc + 1;

        let address = match opcode {
            0x85 => {
                // Zero page
                self.clock += 3;
                self.pc += 2;

                let address = self.system.read_byte(arg_address);
                address as u16
            }
            0x95 => {
                // Zero page, x (zpx)
                self.clock += 4;
                self.pc += 2;

                let address = self.system.read_byte(arg_address) + self.x;
                // TODO: does this wrap around the zero page?
                address as u16
            }
            0x8d => {
                // Absolute (abs)
                self.clock += 4;
                self.pc += 3;

                let address = self.system.read_word(arg_address);
                address
            }
            0x9d => {
                // Absolute, x (abx)
                self.clock += 5;
                self.pc += 3;

                let address = self.system.read_word(arg_address);
                address + self.x as u16
            }
            0x99 => {
                // Absolute, y (aby)
                self.clock += 5;
                self.pc += 3;

                let address = self.system.read_word(arg_address);
                address + self.y as u16
            }
            0x81 => {
                // Indirect zero page, x (izx)
                self.clock += 6;
                self.pc += 2;

                let address = self.system.read_byte(arg_address) + self.x;
                address as u16
            }
            0x91 => {
                // Indirect zero page, y (izy)
                self.clock += 6;
                self.pc += 2;

                let address = self.system.read_byte(arg_address);
                let new_address = self.system.read_word(address as u16) + self.y as u16;
                new_address
            }
            _ => {
                panic!("Unknown opcode");
            }
        };

        self.system.write_byte(address, self.a);
    }

    /// STore X register
    fn stx(&mut self, opcode: u8) {
        let arg_address = self.pc + 1;

        let address = match opcode {
            0x86 => {
                // Zero page
                self.clock += 3;
                self.pc += 2;

                let address = self.system.read_byte(arg_address);
                address as u16
            }
            0x96 => {
                // Zero page, y (zpy)
                self.clock += 4;
                self.pc += 2;

                let address = self.system.read_byte(arg_address) + self.y;
                // TODO: does this wrap around the zero page?
                address as u16
            }
            0x8e => {
                // Absolute (abs)
                self.clock += 4;
                self.pc += 3;

                let address = self.system.read_word(arg_address);
                address
            }
            _ => {
                panic!("Unknown opcode");
            }
        };

        self.system.write_byte(address, self.x);
    }

    /// STore Y register
    fn sty(&mut self, opcode: u8) {
        let arg_address = self.pc + 1;

        let address = match opcode {
            0x84 => {
                // Zero page
                self.clock += 3;
                self.pc += 2;

                let address = self.system.read_byte(arg_address);
                address as u16
            }
            0x94 => {
                // Zero page, x (zpx)
                self.clock += 4;
                self.pc += 2;

                let address = self.system.read_byte(arg_address) + self.x;
                // TODO: does this wrap around the zero page?
                address as u16
            }
            0x8c => {
                // Absolute (abs)
                self.clock += 4;
                self.pc += 3;

                let address = self.system.read_word(arg_address);
                address
            }
            _ => {
                panic!("Unknown opcode");
            }
        };

        self.system.write_byte(address, self.y);
    }
}
