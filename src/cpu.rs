use std::fmt::Display;

use crate::cart::CartLoadResult;
use crate::system::System;

/// The 2A03 NES CPU core, which is based on the 6502 processor
///
/// See: <https://www.nesdev.org/wiki/CPU_registers>
#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
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
    /// See: <https://www.nesdev.org/wiki/Status_flags>
    carry: bool,
    zero: bool,
    interrupt_disable: bool,
    decimal: bool,
    break_flag: bool,
    overflow: bool,
    negative: bool,

    /// System
    system: System,

    /// Clock
    clock: u64,

    /// Helper for storing debug state
    debug_state: String,
    debug_enabled: bool,
}

impl CPU {
    /// Create a new CPU, in the power up state
    ///
    /// See: <https://www.nesdev.org/wiki/CPU_power_up_state>
    pub fn new(filename: String, debug_enabled: bool) -> CartLoadResult<Self> {
        let system = System::new(filename)?;
        let reset_vector = system.read_word(0xfffc);

        Ok(Self {
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
            debug_state: "".to_string(), // this should always be updated before debugging anyway
            debug_enabled,
        })
    }

    fn save_debug_state(&mut self) {
        if !self.debug_enabled {
            return;
        }

        let counters = format!(
            "{:04x}    a: {:02x} x: {:02x} y: {:02x} s: {:02x}",
            self.pc, self.a, self.x, self.y, self.s
        );
        let flags = format!(
            "{}{}{}{}{}{}",
            if self.negative { "N" } else { "-" },
            if self.overflow { "V" } else { "-" },
            if self.decimal { "D" } else { "-" },
            if self.interrupt_disable { "I" } else { "-" },
            if self.zero { "Z" } else { "-" },
            if self.carry { "C" } else { "-" }
        );
        self.debug_state = format!("{counters}    {flags}");
    }

    #[inline]
    fn debug_opcode<S: Into<String> + Display>(&self, opcode_info: S) {
        if !self.debug_enabled {
            return;
        }
        println!("{}    {}", self.debug_state, opcode_info);
    }

    #[inline]
    fn debug_opcode_with_address(&self, opcode_name: &str, address: u16) {
        self.debug_opcode(format!("{} ${:0>4x}", opcode_name, address));
    }

    pub fn run_opcode(&mut self) {
        // Save debug state before altering the counters/registers
        self.save_debug_state();

        let opcode = self.system.read_byte(self.pc);
        match opcode {
            0x00 => self.brk(),
            0x01 => self.ora(opcode),
            0x04 => self.nop(),
            0x05 => self.ora(opcode),
            0x06 => self.asl(opcode),
            0x08 => self.php(),
            0x0c => self.nop(),
            0x0d => self.ora(opcode),
            0x0e => self.asl(opcode),

            0x10 => self.bpl(),
            0x11 => self.ora(opcode),
            0x14 => self.nop(),
            0x15 => self.ora(opcode),
            0x16 => self.asl(opcode),
            0x18 => self.clc(),
            0x19 => self.ora(opcode),
            0x1a => self.nop(),
            0x1c => self.nop(),
            0x1d => self.ora(opcode),
            0x1e => self.asl(opcode),

            0x20 => self.jsr(),
            0x21 => self.and(opcode),
            0x24 => self.bit(opcode),
            0x25 => self.and(opcode),
            0x26 => self.rol(opcode),
            0x28 => self.plp(),
            0x29 => self.and(opcode),
            0x2a => self.rol(opcode),
            0x2c => self.bit(opcode),
            0x2d => self.and(opcode),
            0x2e => self.rol(opcode),

            0x30 => self.bmi(),
            0x31 => self.and(opcode),
            0x34 => self.nop(),
            0x35 => self.and(opcode),
            0x36 => self.rol(opcode),
            0x38 => self.sec(),
            0x39 => self.and(opcode),
            0x3a => self.nop(),
            0x3c => self.nop(),
            0x3d => self.and(opcode),
            0x3e => self.rol(opcode),

            0x40 => self.rti(),
            0x41 => self.eor(opcode),
            0x44 => self.nop(),
            0x45 => self.eor(opcode),
            0x46 => self.rol(opcode),
            0x48 => self.pha(),
            0x49 => self.eor(opcode),
            0x4a => self.rol(opcode),
            0x4c => self.bit(opcode),
            0x4d => self.and(opcode),
            0x4e => self.rol(opcode),

            0x50 => self.bvc(),
            0x51 => self.eor(opcode),
            0x54 => self.nop(),
            0x55 => self.eor(opcode),
            0x56 => self.lsr(opcode),
            0x58 => self.cli(),
            0x59 => self.eor(opcode),
            0x5a => self.nop(),
            0x5c => self.nop(),
            0x5d => self.eor(opcode),
            0x5e => self.lsr(opcode),

            0x60 => self.rts(),
            0x61 => self.adc(opcode),
            0x64 => self.nop(),
            0x65 => self.adc(opcode),
            0x66 => self.ror(opcode),
            0x68 => self.pla(),
            0x69 => self.adc(opcode),
            0x6a => self.ror(opcode),
            0x6c => self.jmp(opcode),
            0x6d => self.adc(opcode),
            0x6e => self.ror(opcode),

            0x70 => self.bvs(),
            0x71 => self.adc(opcode),
            0x74 => self.nop(),
            0x75 => self.adc(opcode),
            0x76 => self.ror(opcode),
            0x78 => self.sei(),
            0x79 => self.adc(opcode),
            0x7a => self.nop(),
            0x7c => self.nop(),
            0x7d => self.adc(opcode),
            0x7e => self.ror(opcode),

            0x80 => self.nop(),
            0x81 => self.sta(opcode),
            0x82 => self.nop(),
            0x84 => self.sty(opcode),
            0x85 => self.sta(opcode),
            0x86 => self.stx(opcode),
            0x88 => self.dey(),
            0x89 => self.nop(),
            0x8a => self.txa(),
            0x8c => self.sty(opcode),
            0x8d => self.sta(opcode),
            0x8e => self.stx(opcode),

            0x90 => self.bcc(),
            0x91 => self.sta(opcode),
            0x94 => self.sty(opcode),
            0x95 => self.sta(opcode),
            0x96 => self.stx(opcode),
            0x98 => self.tya(),
            0x99 => self.sta(opcode),
            0x9a => self.txs(),
            0x9d => self.sta(opcode),

            0xa0 => self.ldy(opcode),
            0xa1 => self.lda(opcode),
            0xa2 => self.ldx(opcode),
            0xa4 => self.ldy(opcode),
            0xa5 => self.lda(opcode),
            0xa6 => self.ldx(opcode),
            0xa8 => self.tay(),
            0xa9 => self.lda(opcode),
            0xaa => self.tax(),
            0xac => self.ldy(opcode),
            0xad => self.lda(opcode),
            0xae => self.ldx(opcode),

            0xb0 => self.bcs(),
            0xb1 => self.lda(opcode),
            0xb4 => self.ldy(opcode),
            0xb5 => self.lda(opcode),
            0xb6 => self.ldx(opcode),
            0xb8 => self.clv(),
            0xb9 => self.lda(opcode),
            0xba => self.tsx(),
            0xbc => self.ldy(opcode),
            0xbd => self.lda(opcode),
            0xbe => self.ldx(opcode),

            0xc0 => self.cpy(opcode),
            0xc1 => self.cmp(opcode),
            0xc2 => self.nop(),
            0xc4 => self.cpy(opcode),
            0xc5 => self.cmp(opcode),
            0xc6 => self.dec(opcode),
            0xc8 => self.iny(),
            0xc9 => self.cmp(opcode),
            0xca => self.dex(),
            0xcc => self.cpy(opcode),
            0xcd => self.cmp(opcode),
            0xce => self.dec(opcode),

            0xd0 => self.bne(),
            0xd1 => self.cmp(opcode),
            0xd4 => self.nop(),
            0xd5 => self.cmp(opcode),
            0xd6 => self.dec(opcode),
            0xd8 => self.cld(),
            0xd9 => self.cmp(opcode),
            0xda => self.nop(),
            0xdc => self.nop(),
            0xdd => self.cmp(opcode),
            0xde => self.dec(opcode),

            0xe0 => self.cpx(opcode),
            0xe1 => self.sbc(opcode),
            0xe2 => self.nop(),
            0xe4 => self.cpx(opcode),
            0xe5 => self.sbc(opcode),
            0xe6 => self.inc(opcode),
            0xe8 => self.inx(),
            0xe9 => self.sbc(opcode),
            0xea => self.nop(),
            0xec => self.cpx(opcode),
            0xed => self.sbc(opcode),
            0xee => self.inc(opcode),

            0xf0 => self.beq(),
            0xf1 => self.sbc(opcode),
            0xf4 => self.nop(),
            0xf5 => self.sbc(opcode),
            0xf6 => self.inc(opcode),
            0xf8 => self.sed(),
            0xf9 => self.sbc(opcode),
            0xfa => self.nop(),
            0xfc => self.nop(),
            0xfd => self.sbc(opcode),
            0xfe => self.inc(opcode),

            _ => panic!("Unknown opcode {:02x}", opcode),
        }
    }

    // Addressing modes --------------------------------------------------------------------------
    fn immediate(&self) -> u16 {
        self.pc + 1
    }

    fn general_zero_page(&self, to_add: u8) -> u16 {
        let next_address = self.immediate();
        (self.system.read_byte(next_address) + to_add) as u16
    }

    fn zero_page(&self) -> u16 {
        self.general_zero_page(0)
    }

    fn zero_page_x(&self) -> u16 {
        self.general_zero_page(self.x)
    }

    fn zero_page_y(&self) -> u16 {
        self.general_zero_page(self.y)
    }

    fn indirect_zero_page_x(&self) -> u16 {
        self.system.read_word(self.zero_page_x())
    }

    fn indirect_zero_page_y(&mut self, extra_clock_for_page_fault: bool) -> u16 {
        let address = self.zero_page();

        let pre_index = self.system.read_word(address);
        let page1 = pre_index >> 8;
        let indirect_address = pre_index + self.y as u16;
        let page2 = indirect_address >> 8;
        if extra_clock_for_page_fault && page1 != page2 {
            self.clock += 1;
        }

        indirect_address
    }

    fn absolute(&self) -> u16 {
        let next_address = self.immediate();
        self.system.read_word(next_address)
    }

    fn absolute_x(&mut self, extra_clock_for_page_fault: bool) -> u16 {
        let mut address = self.absolute();
        let page1 = address >> 8;

        address += self.x as u16;
        let page2 = address >> 8;
        if extra_clock_for_page_fault && page1 != page2 {
            self.clock += 1;
        }

        address
    }

    fn absolute_y(&mut self, extra_clock_for_page_fault: bool) -> u16 {
        let mut address = self.absolute();
        let page1 = address >> 8;

        address += self.y as u16;
        let page2 = address >> 8;
        if extra_clock_for_page_fault && page1 != page2 {
            self.clock += 1;
        }

        address
    }

    // Helpers for setting flags -----------------------------------------------------------------
    fn test_negative(&mut self, value: u8) {
        self.negative = value & 0x80 == 0x80;
    }

    fn test_zero(&mut self, value: u8) {
        self.zero = value == 0;
    }

    // Logical and arithmetic commands -----------------------------------------------------------
    /// bitwise OR with Accumulator
    fn ora(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0x09 => (self.immediate(), 2, 2),
            0x05 => (self.zero_page(), 3, 2),
            0x15 => (self.zero_page_x(), 4, 2),
            0x01 => (self.indirect_zero_page_x(), 6, 2),
            0x11 => (self.indirect_zero_page_y(true), 5, 2),
            0x0d => (self.absolute(), 4, 3),
            0x1d => (self.absolute_x(true), 4, 3),
            0x19 => (self.absolute_y(true), 4, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("ora", intermediate_address);

        self.a |= self.system.read_byte(intermediate_address);
        self.test_negative(self.a);
        self.test_zero(self.a);
    }

    /// bitwise AND with accumulator
    fn and(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0x29 => (self.immediate(), 2, 2),
            0x25 => (self.zero_page(), 3, 2),
            0x35 => (self.zero_page_x(), 4, 2),
            0x21 => (self.indirect_zero_page_x(), 6, 2),
            0x31 => (self.indirect_zero_page_y(true), 5, 2),
            0x2d => (self.absolute(), 4, 3),
            0x3d => (self.absolute_x(true), 4, 3),
            0x39 => (self.absolute_y(true), 4, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode(format!("and {}", intermediate_address));

        self.a &= self.system.read_byte(intermediate_address);
        self.test_negative(self.a);
        self.test_zero(self.a);
    }

    /// bitwise Exclusive OR
    fn eor(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0x49 => (self.immediate(), 2, 2),
            0x45 => (self.zero_page(), 3, 2),
            0x55 => (self.zero_page_x(), 4, 2),
            0x41 => (self.indirect_zero_page_x(), 6, 2),
            0x51 => (self.indirect_zero_page_y(true), 5, 2),
            0x4d => (self.absolute(), 4, 3),
            0x5d => (self.absolute_x(true), 4, 3),
            0x59 => (self.absolute_y(true), 4, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("eor", intermediate_address);

        self.a ^= self.system.read_byte(intermediate_address);
        self.test_negative(self.a);
        self.test_zero(self.a);
    }

    /// ADd with Carry
    fn adc(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0x69 => (self.immediate(), 2, 2),
            0x65 => (self.zero_page(), 3, 2),
            0x75 => (self.zero_page_x(), 4, 2),
            0x61 => (self.indirect_zero_page_x(), 6, 2),
            0x71 => (self.indirect_zero_page_y(true), 5, 2),
            0x6d => (self.absolute(), 4, 3),
            0x7d => (self.absolute_x(true), 4, 3),
            0x79 => (self.absolute_y(true), 4, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("adc", intermediate_address);

        let intermediate =
            self.a as i16 + self.system.read_byte(intermediate_address) as i16 + !self.carry as i16;
        self.overflow = !(-128..=127).contains(&intermediate);
        self.carry = (intermediate as u16) & 0xff00 != 0;
        self.a = intermediate as u8;

        self.test_negative(self.a);
        self.test_zero(self.a);
    }

    /// SuBtract with Carry
    fn sbc(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0xe9 => (self.immediate(), 2, 2),
            0xe5 => (self.zero_page(), 3, 2),
            0xf5 => (self.zero_page_x(), 4, 2),
            0xe1 => (self.indirect_zero_page_x(), 6, 2),
            0xf1 => (self.indirect_zero_page_y(true), 5, 2),
            0xed => (self.absolute(), 4, 3),
            0xfd => (self.absolute_x(true), 4, 3),
            0xf9 => (self.absolute_y(true), 4, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("sbc", intermediate_address);

        let intermediate =
            self.a as i16 - self.system.read_byte(intermediate_address) as i16 - !self.carry as i16;
        self.overflow = !(-128..=127).contains(&intermediate);
        self.carry = (intermediate as u16) & 0xff00 != 0;
        self.a = intermediate as u8;

        self.test_negative(self.a);
        self.test_zero(self.a);
    }

    /// CoMPare accumulator
    fn cmp(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0xc9 => (self.immediate(), 2, 2),
            0xc5 => (self.zero_page(), 3, 2),
            0xd5 => (self.zero_page_x(), 4, 2),
            0xc1 => (self.indirect_zero_page_x(), 6, 2),
            0xd1 => (self.indirect_zero_page_y(true), 5, 2),
            0xcd => (self.absolute(), 4, 3),
            0xdd => (self.absolute_x(true), 4, 3),
            0xd9 => (self.absolute_y(true), 4, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("cmp", intermediate_address);

        let intermediate = self.a as i16 - self.system.read_byte(intermediate_address) as i16;
        self.negative = (intermediate & 0x80) == 0x80;
        self.zero = intermediate == 0;
        self.carry = intermediate >= 0;
    }

    /// ComPare X register
    fn cpx(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0xc0 => (self.immediate(), 2, 2),
            0xc4 => (self.zero_page(), 3, 2),
            0xcc => (self.absolute(), 4, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("cpx", intermediate_address);

        let intermediate = self.y as i16 - self.system.read_byte(intermediate_address) as i16;
        self.negative = intermediate & 0x80 == 0x80;
        self.zero = intermediate == 0;
        self.carry = intermediate >= 0;
    }

    /// ComPare Y register
    fn cpy(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0xe0 => (self.immediate(), 2, 2),
            0xe4 => (self.zero_page(), 3, 2),
            0xec => (self.absolute(), 4, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("cpy", intermediate_address);

        let intermediate = self.x as i16 - self.system.read_byte(intermediate_address) as i16;
        self.negative = intermediate & 0x80 == 0x80;
        self.zero = intermediate == 0;
        self.carry = intermediate >= 0;
    }

    /// DECrement memory
    fn dec(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0xc6 => (self.zero_page(), 5, 2),
            0xd6 => (self.zero_page_x(), 6, 2),
            0xce => (self.absolute(), 6, 3),
            0xde => (self.absolute_x(false), 7, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("dec", intermediate_address);

        let intermediate = self.system.read_byte(intermediate_address) - 1;
        self.test_negative(intermediate);
        self.test_zero(intermediate);
        self.system.write_byte(intermediate_address, intermediate);
    }

    /// DEcrement X
    fn dex(&mut self) {
        self.debug_opcode("dex");

        self.clock += 2;
        self.pc += 1;

        self.x -= 1;
        self.test_negative(self.x);
        self.test_zero(self.x);
    }

    /// DEcrement Y
    fn dey(&mut self) {
        self.debug_opcode("dey");

        self.clock += 2;
        self.pc += 1;

        self.y -= 1;
        self.test_negative(self.y);
        self.test_zero(self.y);
    }

    /// INCrement memory
    fn inc(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0xe6 => (self.zero_page(), 5, 2),
            0xf6 => (self.zero_page_x(), 6, 2),
            0xee => (self.absolute(), 6, 3),
            0xfe => (self.absolute_x(false), 7, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("inc", intermediate_address);

        let intermediate = self.system.read_byte(intermediate_address) + 1;
        self.test_negative(intermediate);
        self.test_zero(intermediate);
        self.system.write_byte(intermediate_address, intermediate);
    }

    /// INcrement X
    fn inx(&mut self) {
        self.debug_opcode("inc");

        self.clock += 2;
        self.pc += 1;

        self.x += 1;
        self.test_negative(self.x);
        self.test_zero(self.x);
    }

    /// INcrement Y
    fn iny(&mut self) {
        self.debug_opcode("iny");

        self.clock += 2;
        self.pc += 1;

        self.y += 1;
        self.test_negative(self.y);
        self.test_zero(self.y);
    }

    /// Arithmetic Shift Left
    fn asl(&mut self, opcode: u8) {
        // Dealing with the accumulator directly doesn't fit the pattern well, so handle separately
        if opcode == 0x0a {
            self.debug_opcode("asl A");

            self.carry = self.a & 0x80 == 0x80;
            self.a <<= 1;
            self.test_negative(self.a);
            self.test_zero(self.a);
            self.clock += 2;
            self.pc += 1;
            return;
        }

        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0x06 => (self.zero_page(), 5, 2),
            0x16 => (self.zero_page_x(), 6, 2),
            0x0e => (self.absolute(), 6, 3),
            0x1e => (self.absolute_x(false), 7, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("asl {}", intermediate_address);

        let mut intermediate = self.system.read_byte(intermediate_address);
        self.carry = (intermediate & 0x80) == 0x80;
        intermediate <<= 1;
        self.test_negative(intermediate);
        self.test_zero(intermediate);
        self.system.write_byte(intermediate_address, intermediate);
    }

    /// ROtate Left
    fn rol(&mut self, opcode: u8) {
        let carry_value = self.carry as u8;

        // Dealing with the accumulator directly doesn't fit the pattern well, so handle separately
        if opcode == 0x2a {
            self.debug_opcode("rol A");

            self.carry = self.a & 0x80 == 0x80;
            self.a <<= 1 + carry_value;
            self.test_negative(self.a);
            self.test_zero(self.a);
            self.clock += 2;
            self.pc += 1;
            return;
        }

        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0x26 => (self.zero_page(), 5, 2),
            0x36 => (self.zero_page_x(), 6, 2),
            0x2e => (self.absolute(), 6, 3),
            0x3e => (self.absolute_x(false), 7, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("rol {}", intermediate_address);

        let mut intermediate = self.system.read_byte(intermediate_address);
        self.carry = (intermediate & 0x80) == 0x80;
        intermediate <<= 1 + carry_value;
        self.test_negative(intermediate);
        self.test_zero(intermediate);
        self.system.write_byte(intermediate_address, intermediate);
    }

    ///Logical Shift Right
    fn lsr(&mut self, opcode: u8) {
        // Dealing with the accumulator directly doesn't fit the pattern well, so handle separately
        if opcode == 0x4a {
            self.debug_opcode("lsr A");

            self.carry = self.a & 0x01 == 0x01;
            self.a >>= 1;
            self.test_negative(self.a);
            self.test_zero(self.a);
            self.clock += 2;
            self.pc += 1;
            return;
        }

        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0x46 => (self.zero_page(), 5, 2),
            0x56 => (self.zero_page_x(), 6, 2),
            0x4e => (self.absolute(), 6, 3),
            0x5e => (self.absolute_x(false), 7, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("lsr {}", intermediate_address);

        let mut intermediate = self.system.read_byte(intermediate_address);
        self.carry = (intermediate & 0x01) == 0x01;
        intermediate >>= 1;
        self.test_negative(intermediate);
        self.test_zero(intermediate);
        self.system.write_byte(intermediate_address, intermediate);
    }

    /// ROtate Right
    fn ror(&mut self, opcode: u8) {
        let carry_value: u8 = if self.carry { 0x80 } else { 0 };

        // Dealing with the accumulator directly doesn't fit the pattern well, so handle separately
        if opcode == 0x6a {
            self.debug_opcode("ror A");

            self.carry = self.a & 0x01 == 0x01;
            self.a >>= 1;
            self.test_negative(self.a);
            self.test_zero(self.a);
            self.clock += 2;
            self.pc += 1;
            return;
        }

        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0x66 => (self.zero_page(), 5, 2),
            0x76 => (self.zero_page_x(), 6, 2),
            0x6e => (self.absolute(), 6, 3),
            0x7e => (self.absolute_x(false), 7, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("ror", intermediate_address);

        let mut intermediate = self.system.read_byte(intermediate_address);
        self.carry = (intermediate & 0x01) == 0x01;
        intermediate >>= 1 + carry_value;
        self.test_negative(intermediate);
        self.test_zero(intermediate);
        self.system.write_byte(intermediate_address, intermediate);
    }

    // Move commands -----------------------------------------------------------------------------
    /// LoaD Accumulator
    fn lda(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0xa9 => (self.immediate(), 2, 2),
            0xa5 => (self.zero_page(), 3, 2),
            0xb5 => (self.zero_page_x(), 4, 2),
            0xad => (self.absolute(), 4, 3),
            0xbd => (self.absolute_x(true), 6, 3),
            0xb9 => (self.absolute_y(true), 4, 2),
            0xa1 => (self.indirect_zero_page_x(), 6, 4),
            0xb1 => (self.indirect_zero_page_y(true), 6, 2),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("lda", intermediate_address);

        let intermediate = self.system.read_byte(intermediate_address);
        self.test_negative(intermediate);
        self.test_zero(intermediate);

        self.a = intermediate;
    }

    /// LoaD X register
    fn ldx(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0xa2 => (self.immediate(), 2, 2),
            0xa6 => (self.zero_page(), 3, 2),
            0xb6 => (self.zero_page_y(), 4, 2),
            0xae => (self.absolute(), 4, 3),
            0xbe => (self.absolute_y(true), 4, 2),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("ldx", intermediate_address);

        let intermediate = self.system.read_byte(intermediate_address);
        self.test_negative(intermediate);
        self.test_zero(intermediate);

        self.x = intermediate;
    }

    /// LoaD Y register
    fn ldy(&mut self, opcode: u8) {
        let (intermediate_address, clock_increment, pc_increment) = match opcode {
            0xa0 => (self.immediate(), 2, 2),
            0xa4 => (self.zero_page(), 3, 2),
            0xb4 => (self.zero_page_x(), 4, 2),
            0x8c => (self.absolute(), 4, 3),
            0xbc => (self.absolute_x(true), 4, 2),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("ldy", intermediate_address);

        let intermediate = self.system.read_byte(intermediate_address);
        self.test_negative(intermediate);
        self.test_zero(intermediate);

        self.y = intermediate;
    }

    /// STore Accumulator
    fn sta(&mut self, opcode: u8) {
        let (address, clock_increment, pc_increment) = match opcode {
            0x85 => (self.zero_page(), 3, 2),
            0x95 => (self.zero_page_x(), 4, 2),
            0x8d => (self.absolute(), 4, 3),
            0x9d => (self.absolute_x(false), 5, 3),
            0x99 => (self.absolute_y(false), 5, 3),
            0x81 => (self.indirect_zero_page_x(), 6, 2),
            0x91 => (self.indirect_zero_page_y(false), 6, 2),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("sta", address);

        self.system.write_byte(address, self.a);
    }

    /// STore X register
    fn stx(&mut self, opcode: u8) {
        let (address, clock_increment, pc_increment) = match opcode {
            0x86 => (self.zero_page(), 3, 2),
            0x96 => (self.zero_page_y(), 4, 2),
            0x8e => (self.absolute(), 4, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("stx", address);

        self.system.write_byte(address, self.x);
    }

    /// STore Y register
    fn sty(&mut self, opcode: u8) {
        let (address, clock_increment, pc_increment) = match opcode {
            0x84 => (self.zero_page(), 3, 2),
            0x94 => (self.zero_page_y(), 4, 2),
            0x8c => (self.absolute(), 4, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("sty", address);

        self.system.write_byte(address, self.y);
    }

    /// Transfer A to X
    fn tax(&mut self) {
        self.debug_opcode("tax");

        self.clock += 2;
        self.pc += 1;

        self.test_negative(self.a);
        self.test_zero(self.a);

        self.x = self.a;
    }

    /// Transfer X to A
    fn txa(&mut self) {
        self.debug_opcode("txa");

        self.clock += 2;
        self.pc += 1;

        self.test_negative(self.x);
        self.test_zero(self.x);

        self.a = self.x;
    }

    /// Transfer A to Y
    fn tay(&mut self) {
        self.debug_opcode("tay");

        self.clock += 2;
        self.pc += 1;

        self.test_negative(self.a);
        self.test_zero(self.a);

        self.y = self.a;
    }

    /// Transfer X to A
    fn tya(&mut self) {
        self.debug_opcode("tya");

        self.clock += 2;
        self.pc += 1;

        self.test_negative(self.y);
        self.test_zero(self.y);

        self.a = self.y;
    }

    /// Transfer S to X
    fn tsx(&mut self) {
        self.debug_opcode("tsx");

        self.clock += 2;
        self.pc += 1;

        self.test_negative(self.s);
        self.test_zero(self.s);

        self.x = self.s;
    }

    /// Transfer X to S
    fn txs(&mut self) {
        self.debug_opcode("txs");

        self.clock += 2;
        self.pc += 1;

        self.s = self.x;
    }

    /// PuLl Accumulator
    fn pla(&mut self) {
        self.debug_opcode("pla");

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
        self.debug_opcode("pha");

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
        self.debug_opcode("plp");

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
        self.debug_opcode("php");

        self.clock += 3;
        self.pc += 1;

        self.push_status();
    }

    // Jump/Flag commands ------------------------------------------------------------------------
    /// Common function for branching opcodes. The opcode name is just passed in for debugging.
    fn branch(&mut self, opcode_name: &str) {
        let arg_address = self.immediate();
        let address = self.system.read_byte(arg_address) as i8;

        // For this pc increment, see https://github.com/jntrnr/jaktnesmonster/pull/1
        self.pc += 2;

        let prev_page = self.pc >> 8;
        // TODO: test this
        self.pc = (self.pc as i16 + address as i16) as u16;

        self.debug_opcode_with_address(opcode_name, self.pc);

        let new_page = self.pc >> 8;
        if prev_page != new_page {
            self.clock += 4;
        } else {
            self.clock += 3;
        }
    }

    fn branch_if(&mut self, condition: bool, opcode_name: &str) {
        if condition {
            self.branch(opcode_name);
        } else {
            self.clock += 2;
            self.pc += 2;
        }
    }

    /// Branch on PLus
    fn bpl(&mut self) {
        self.branch_if(!self.negative, "bpl");
    }

    /// Branch on MInus
    fn bmi(&mut self) {
        self.branch_if(self.negative, "bmi");
    }

    /// Branch on oVerflow Clear
    fn bvc(&mut self) {
        self.branch_if(!self.overflow, "bvc");
    }

    /// Branch on oVerflow Set
    fn bvs(&mut self) {
        self.branch_if(self.overflow, "bvs");
    }

    /// Branch on Carry Clear
    fn bcc(&mut self) {
        self.branch_if(!self.carry, "bcc");
    }

    /// Branch on Carry Set
    fn bcs(&mut self) {
        self.branch_if(self.carry, "bcs");
    }

    /// Branch on Not Equal
    fn bne(&mut self) {
        self.branch_if(!self.zero, "bne");
    }

    /// Branch on EQual
    fn beq(&mut self) {
        self.branch_if(self.zero, "beq");
    }

    /// BReaK
    fn brk(&mut self) {
        self.debug_opcode("brk");

        self.clock += 7;

        self.push_word(self.pc);

        let break_address = 0xfffe;
        self.pc = self.system.read_word(break_address);
        self.break_flag = true;
        self.interrupt_disable = true;
    }

    /// ReTurn from Interrupt
    fn rti(&mut self) {
        self.debug_opcode("rti");

        self.clock += 6;
        self.pull_status();
        self.pull_pc();
    }

    /// Jump to SubRoutine
    fn jsr(&mut self) {
        self.debug_opcode("jsr");

        self.clock += 6;

        self.push_word(self.pc + 2);

        let arg_address = self.immediate();
        self.pc = self.system.read_word(arg_address);
    }

    /// ReTurn from Subroutine
    fn rts(&mut self) {
        self.debug_opcode("rts");

        self.clock += 6;
        self.pull_pc()
    }

    /// JuMP
    fn jmp(&mut self, opcode: u8) {
        let (address, clock_increment) = match opcode {
            0x24 => (self.absolute(), 3),
            0x2c => (self.system.read_word(self.absolute()), 5), // Indirect absolute (ind)
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;

        self.debug_opcode_with_address("jmp", address);

        self.pc = address;
    }

    /// test BITs
    fn bit(&mut self, opcode: u8) {
        let (address, clock_increment, pc_increment) = match opcode {
            0x24 => (self.zero_page(), 3, 2),
            0x2c => (self.absolute(), 4, 3),
            _ => panic!("Unknown opcode {:02x}", opcode),
        };
        self.clock += clock_increment;
        self.pc += pc_increment;

        self.debug_opcode_with_address("bit", address);

        let value = self.system.read_byte(address);
        self.zero = value & self.a == 0;
        self.negative = value & 0x80 == 0x80;
        self.overflow = value & 0x40 == 0x40;
    }

    /// CLear Carry
    fn clc(&mut self) {
        self.debug_opcode("clc");

        self.clock += 2;
        self.pc += 1;
        self.carry = false;
    }

    /// SEt Carry
    fn sec(&mut self) {
        self.debug_opcode("sec");

        self.clock += 2;
        self.pc += 1;
        self.carry = true;
    }

    /// CLear Decimal
    fn cld(&mut self) {
        self.debug_opcode("cld");

        self.clock += 2;
        self.pc += 1;
        self.decimal = false;
    }

    // SEt Decimal
    fn sed(&mut self) {
        self.debug_opcode("sed");

        self.clock += 2;
        self.pc += 1;
        self.decimal = true;
    }

    /// CLear Interrupt
    fn cli(&mut self) {
        self.debug_opcode("cli");

        self.clock += 2;
        self.pc += 1;
        self.interrupt_disable = false;
    }

    /// SEt Interrupt
    fn sei(&mut self) {
        self.debug_opcode("sei");

        self.clock += 2;
        self.pc += 1;
        self.interrupt_disable = true;
    }

    /// CLear oVerflow
    fn clv(&mut self) {
        self.debug_opcode("clv");

        self.clock += 2;
        self.pc += 1;
        self.overflow = false;
    }

    /// No OPeration
    fn nop(&mut self) {
        self.debug_opcode("nop");

        self.clock += 2;
        self.pc += 1;
    }
}
