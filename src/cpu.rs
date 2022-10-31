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
    break_flag: bool,
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
    pub fn new(system: &'a mut System) -> Self {
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
            break_flag: false,
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

    // Addressing modes --------------------------------------------------------------------------
    fn immediate(&mut self) -> u8 {
        let arg_address = self.pc + 1;
        self.system.read_byte(arg_address)
    }

    fn zero_page(&mut self) -> u8 {
        let arg_address = self.pc + 1;
        let address = self.system.read_byte(arg_address) as u16;
        self.system.read_byte(address)
    }

    fn zero_page_x(&mut self) -> u8 {
        let arg_address = self.pc + 1;
        let address = (self.system.read_byte(arg_address) + self.x) as u16;
        self.system.read_byte(address)
    }

    fn zero_page_y(&mut self) -> u8 {
        let arg_address = self.pc + 1;
        let address = (self.system.read_byte(arg_address) + self.y) as u16;
        self.system.read_byte(address)
    }

    fn indirect_zero_page_x(&mut self) -> u8 {
        let arg_address = self.pc + 1;
        let address = (self.system.read_byte(arg_address) + self.x) as u16;
        let indirect_address = self.system.read_word(address);
        self.system.read_byte(indirect_address)
    }

    fn indirect_zero_page_y(&mut self, extra_clock_for_page_fault: bool) -> u8 {
        let arg_address = self.pc + 1;
        let address = (self.system.read_byte(arg_address) + self.x) as u16;

        let pre_index = self.system.read_word(address);
        let page1 = pre_index >> 2;
        let indirect_address = pre_index + self.y as u16;
        let page2 = indirect_address >> 2;
        if extra_clock_for_page_fault && page1 != page2 {
            self.clock += 1;
        }

        self.system.read_byte(indirect_address)
    }

    fn absolute(&mut self) -> u8 {
        let arg_address = self.pc + 1;
        let address = self.system.read_word(arg_address);
        self.system.read_byte(address)
    }

    fn absolute_x(&mut self, extra_clock_for_page_fault: bool) -> u8 {
        let arg_address = self.pc + 1;
        let mut address = self.system.read_word(arg_address);
        let page1 = address >> 2;

        address += self.x as u16;
        let page2 = address >> 2;
        if extra_clock_for_page_fault && page1 != page2 {
            self.clock += 1;
        }

        self.system.read_byte(address)
    }

    fn absolute_y(&mut self, extra_clock_for_page_fault: bool) -> u8 {
        let arg_address = self.pc + 1;
        let mut address = self.system.read_word(arg_address);
        let page = address >> 2;

        address += self.y as u16;
        let new_page = address >> 2;
        if page != new_page {
            self.clock += 1;
        }

        self.system.read_byte(address)
    }

    // Helpers for setting flags -----------------------------------------------------------------
    fn test_negative(&mut self, value: u8) {
        self.negative = value & 0x80 == 0x80;
    }

    fn test_zero(&mut self, value: u8) {
        self.zero = value == 0;
    }

    // Move commands -----------------------------------------------------------------------------
    /// LoaD Accumulator
    fn lda(&mut self, opcode: u8) {
        let intermediate = match opcode {
            0xa9 => {
                self.clock += 2;
                self.pc += 2;
                self.immediate()
            }
            0xa5 => {
                self.clock += 3;
                self.pc += 2;
                self.zero_page()
            }
            0xb5 => {
                self.clock += 4;
                self.pc += 2;
                self.zero_page_x()
            }
            0xad => {
                self.clock += 4;
                self.pc += 3;
                self.absolute()
            }
            0xbd => {
                self.clock += 6;
                self.pc += 3;
                self.absolute_x(true)
            }
            0xb9 => {
                self.clock += 4;
                self.pc += 2;
                self.absolute_y(true)
            }
            0xa1 => {
                self.clock += 6;
                self.pc += 4;
                self.indirect_zero_page_x()
            }
            0xb1 => {
                self.clock += 6;
                self.pc += 2;
                self.indirect_zero_page_y(true)
            }
            _ => panic!("Unknown opcode"),
        };

        self.test_negative(intermediate);
        self.test_zero(intermediate);

        self.a = intermediate;
    }

    /// LoaD X register
    fn ldx(&mut self, opcode: u8) {
        let intermediate = match opcode {
            0xa2 => {
                self.clock += 2;
                self.pc += 2;
                self.immediate()
            }
            0xa6 => {
                self.clock += 3;
                self.pc += 2;
                self.zero_page()
            }
            0xb6 => {
                self.clock += 4;
                self.pc += 2;
                self.zero_page_y()
            }
            0xae => {
                self.clock += 4;
                self.pc += 3;
                self.absolute()
            }
            0xbe => {
                self.clock += 4;
                self.pc += 2;
                self.absolute_y(true)
            }
            _ => panic!("Unknown opcode"),
        };

        self.test_negative(intermediate);
        self.test_zero(intermediate);

        self.x = intermediate;
    }

    /// LoaD Y register
    fn ldy(&mut self, opcode: u8) {
        let intermediate = match opcode {
            0xa0 => {
                self.clock += 2;
                self.pc += 2;
                self.immediate()
            }
            0xa4 => {
                self.clock += 3;
                self.pc += 2;
                self.zero_page()
            }
            0xb4 => {
                self.clock += 4;
                self.pc += 2;
                self.zero_page_x()
            }
            0x8c => {
                self.clock += 4;
                self.pc += 3;
                self.absolute()
            }
            0xbc => {
                self.clock += 4;
                self.pc += 2;
                self.absolute_x(true)
            }
            _ => panic!("Unknown opcode"),
        };

        self.test_negative(intermediate);
        self.test_zero(intermediate);

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

    /// Transfer A to X
    fn tax(&mut self) {
        // TODO: verify opcode is $AA?
        self.clock += 2;
        self.pc += 1;

        self.test_negative(self.a);
        self.test_zero(self.a);

        self.x = self.a;
    }

    /// Transfer X to A
    fn txa(&mut self) {
        self.clock += 2;
        self.pc += 1;

        self.test_negative(self.x);
        self.test_zero(self.x);

        self.a = self.x;
    }

    /// Transfer A to Y
    fn tay(&mut self) {
        self.clock += 2;
        self.pc += 1;

        self.test_negative(self.a);
        self.test_zero(self.a);

        self.y = self.a;
    }

    /// Transfer X to A
    fn tya(&mut self) {
        self.clock += 2;
        self.pc += 1;

        self.test_negative(self.y);
        self.test_zero(self.y);

        self.a = self.y;
    }

    /// Transfer S to X
    fn tsx(&mut self) {
        self.clock += 2;
        self.pc += 1;

        self.test_negative(self.s);
        self.test_zero(self.s);

        self.x = self.s;
    }

    /// Transfer X to S
    fn txs(&mut self) {
        self.clock += 2;
        self.pc += 1;

        self.s = self.x;
    }

    /// PuLl Accumulator
    fn pla(&mut self) {
        self.clock += 4;
        self.pc += 1;

        self.s += 1;
        let intermediate = self.system.read_byte(0x100 + self.s as u16);

        self.test_negative(intermediate);
        self.test_zero(intermediate);

        self.a = intermediate;
    }

    /// PusH Accumulator
    fn pha(&mut self) {
        self.clock += 3;
        self.pc += 1;

        self.system.write_byte(0x100 + self.s as u16, self.a);
        self.s -= 1;
    }

    /// Pull status from System
    fn pull_status(&mut self) {
        self.s += 1;
        let intermediate = self.system.read_byte(0x100 + self.s as u16);

        self.negative = intermediate & 0x80 == 0x80;
        self.overflow = intermediate & 0x40 == 0x40;
        self.decimal = intermediate & 0x08 == 0x08;
        self.interrupt_disable = intermediate & 0x04 == 0x04;
        self.zero = intermediate & 0x02 == 0x02;
        self.carry = intermediate & 0x01 == 0x01;
    }

    /// Pull program counter
    fn pull_pc(&mut self) {
        self.s += 1;
        self.system.read_word(0x100u16 + self.s as u16);
        self.s += 1;
    }

    /// PuLl Processor status
    fn plp(&mut self) {
        self.clock += 4;
        self.pc += 1;

        self.pull_status();
    }

    /// Push status to System
    fn push_status(&mut self) {
        let mut intermediate: u8 = 0;
        if self.negative {
            intermediate |= 0x80;
        }
        if self.overflow {
            intermediate |= 0x40;
        }
        intermediate |= 0x02; // always 1
        if self.break_flag {
            intermediate |= 0x10;
        }
        if self.decimal {
            intermediate |= 0x08;
        }
        if self.interrupt_disable {
            intermediate |= 0x04;
        }
        if self.zero {
            intermediate |= 0x02;
        }
        if self.carry {
            intermediate |= 0x01;
        }

        self.system.write_byte(0x100 + self.s as u16, intermediate);
        self.s -= 1;
    }

    /// Push word to System
    fn push_word(&mut self, value: u16) {
        // TODO: What order should this push the bytes?
        let first_byte = (value >> 8) as u8;
        self.system.write_byte(0x100u16 + self.s as u16, first_byte);
        self.s -= 1;

        let second_byte = (value & 0xff) as u8;
        self.system
            .write_byte(0x100u16 + self.s as u16, second_byte);
        self.s -= 1;
    }

    /// PusH Processor status
    fn php(&mut self) {
        self.clock += 3;
        self.pc += 1;

        self.push_status();
    }

    fn branch(&mut self) {
        let arg_address = self.pc + 1;
        let address = self.system.read_byte(arg_address) as i8;

        let prev_page = self.pc >> 8;
        // TODO: test this
        self.pc = (self.pc as i16 + address as i16) as u16;
        let new_page = self.pc >> 8;
        if prev_page != new_page {
            self.clock += 4;
        } else {
            self.clock += 3;
        }
    }

    fn branch_if(&mut self, condition: bool) {
        if condition {
            self.branch();
        } else {
            self.clock += 2;
            self.pc += 2;
        }
    }

    /// Branch on PLus
    fn bpl(&mut self) {
        self.branch_if(!self.negative);
    }

    /// Branch on MInus
    fn bmi(&mut self) {
        self.branch_if(self.negative);
    }

    /// Branch on oVerflow Clear
    fn bvc(&mut self) {
        self.branch_if(!self.overflow);
    }

    /// Branch on oVerflow Set
    fn bvs(&mut self) {
        self.branch_if(self.overflow);
    }

    /// Branch on Carry Clear
    fn bcc(&mut self) {
        self.branch_if(!self.carry);
    }

    /// Branch on Carry Set
    fn bcs(&mut self) {
        self.branch_if(self.carry);
    }

    /// Branch on Not Equal
    fn bne(&mut self) {
        self.branch_if(!self.zero);
    }

    /// Branch on EQual
    fn beq(&mut self) {
        self.branch_if(self.zero);
    }

    /// BReaK
    fn brk(&mut self) {
        self.clock += 7;

        self.push_word(self.pc);

        let break_address = 0xfffe;
        self.pc = self.system.read_word(break_address);
        self.break_flag = true;
        self.interrupt_disable = true;
    }

    /// ReTurn from Interrupt
    fn rti(&mut self) {
        self.clock += 6;
        self.pull_status();
        self.pull_pc();
    }

    /// Jump to SubRoutine
    fn jsr(&mut self) {
        self.clock += 6;

        self.push_word(self.pc + 2);

        let arg_address = self.pc + 1;
        let address = self.system.read_word(arg_address);
    }

    /// ReTurn from Subroutine
    fn rts(&mut self) {
        self.clock += 6;
        self.pull_pc()
    }

    /// JuMP
    fn jmp(&mut self, opcode: u8) {
        let arg_address = self.pc + 1;

        match opcode {
            0x24 => {
                // Absolute (abs)
                self.clock += 3;
                let address = self.system.read_word(arg_address);
                self.pc = address;
            }
            0x2c => {
                // Indirect absolute (ind)
                self.clock += 5;
                let indirect_address = self.system.read_word(arg_address);
                let address = self.system.read_word(indirect_address);
                self.pc = address;
            }
            _ => panic!("Unknown opcode"),
        }
    }

    /// test BITs
    fn bit(&mut self, opcode: u8) {
        let arg_address = self.pc + 1;

        let value = match opcode {
            0x24 => {
                // Zero page (zp)
                self.clock += 3;
                self.pc += 2;

                let address = self.system.read_byte(arg_address);
                self.system.read_byte(address as u16)
            }
            0x2c => {
                // Absolute (abs)
                self.clock += 4;
                self.pc += 3;

                let address = self.system.read_word(arg_address);
                self.system.read_byte(address)
            }
            _ => panic!("Unknown opcode"),
        };

        self.zero = value & self.a == 0;
        self.negative = value & 0x80 == 0x80;
        self.overflow = value & 0x40 == 0x40;
    }

    /// CLear Carry
    fn clc(&mut self) {
        self.clock += 2;
        self.pc += 1;
        self.carry = false;
    }

    /// SEt Carry
    fn sec(&mut self) {
        self.clock += 2;
        self.pc += 1;
        self.carry = true;
    }

    /// CLear Decimal
    fn cld(&mut self) {
        self.clock += 2;
        self.pc += 1;
        self.decimal = false;
    }

    // SEt Decimal
    fn sed(&mut self) {
        self.clock += 2;
        self.pc += 1;
        self.decimal = true;
    }

    /// CLear Interrupt
    fn cli(&mut self) {
        self.clock += 2;
        self.pc += 1;
        self.interrupt_disable = false;
    }

    /// SEt Interrupt
    fn sei(&mut self) {
        self.clock += 2;
        self.pc += 1;
        self.interrupt_disable = true;
    }

    /// CLear oVerflow
    fn clv(&mut self) {
        self.clock += 2;
        self.pc += 1;
        self.overflow = false;
    }

    /// No OPeration
    fn nop(&mut self) {
        self.clock += 2;
        self.pc += 1;
    }
}
