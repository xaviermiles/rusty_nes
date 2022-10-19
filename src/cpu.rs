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
    system: &'a System,
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
        }
    }
}
