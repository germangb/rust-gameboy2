use crate::{device::MainDevice, error::Error};
pub use registers::Registers;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[macro_use]
mod registers;
mod cycles;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CPU {
    registers: Registers,
    ime: bool,
    halt: bool,
}

impl CPU {
    /// Returns the current state of the CPU registers.
    pub fn reg(&self) -> &Registers {
        &self.registers
    }

    // not exposed because we don't want users messing with registers...
    // for now...
    pub(crate) fn reg_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }

    /// Returns true id the CPU is currently halted, or false otherwise.
    /// If halted, the CPU will remain so until the next interrupt is
    /// acknowledged.
    pub fn halt(&self) -> bool {
        self.halt
    }

    /// Returns true if the main interrupt switch is enabled, or false
    /// otherwise. If it's disabled, none of the interrupts will be
    /// acknowledged.
    pub fn ime(&self) -> bool {
        self.ime
    }

    pub(super) fn update<D: MainDevice>(&mut self, device: &mut D) -> Result<u64, Error> {
        let int = self.int(device)?;
        let cycles = if int != 0 {
            int
        } else if !self.halt {
            self.exec(device)?
        } else {
            4
        };

        Ok(cycles)
    }

    fn int<D: MainDevice>(&mut self, device: &mut D) -> Result<u64, Error> {
        let ie = <D as MainDevice>::read(device, 0xffff)?;
        let if_ = <D as MainDevice>::read(device, 0xff0f)?;
        let tr = (ie & if_).trailing_zeros() as u8;
        if tr <= 4 {
            self.halt = false;
        }
        if !self.ime || tr > 4 {
            return Ok(0);
        }
        self.int_v([0x40, 0x48, 0x50, 0x58, 0x60][tr as usize], device)?;
        self.ime = false;
        <D as MainDevice>::write(device, 0xff0f, if_ & !(1 << tr))?;
        Ok(cycles::unprefixed(0, false)) // NOP
    }

    fn int_v<D: MainDevice>(&mut self, v: u16, device: &mut D) -> Result<(), Error> {
        self.stack_push(self.registers.pc, device)?;
        self.registers.pc = v;
        Ok(())
    }

    fn exec_opcode_cb<D: MainDevice>(&mut self, device: &mut D, opcode: u8) -> Result<u64, Error> {
        match opcode {
            // RLC n
            0x00 => self.registers.b = self.rlc_n(self.registers.b),
            0x01 => self.registers.c = self.rlc_n(self.registers.c),
            0x02 => self.registers.d = self.rlc_n(self.registers.d),
            0x03 => self.registers.e = self.rlc_n(self.registers.e),
            0x04 => self.registers.h = self.rlc_n(self.registers.h),
            0x05 => self.registers.l = self.rlc_n(self.registers.l),
            0x06 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.rlc_n(<D as MainDevice>::read(device, hl)?),
                )?
            }
            0x07 => self.registers.a = self.rlc_n(self.registers.a),

            // RRC n
            0x08 => self.registers.b = self.rrc_n(self.registers.b),
            0x09 => self.registers.c = self.rrc_n(self.registers.c),
            0x0a => self.registers.d = self.rrc_n(self.registers.d),
            0x0b => self.registers.e = self.rrc_n(self.registers.e),
            0x0c => self.registers.h = self.rrc_n(self.registers.h),
            0x0d => self.registers.l = self.rrc_n(self.registers.l),
            0x0e => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.rrc_n(<D as MainDevice>::read(device, hl)?),
                )?
            }
            0x0f => self.registers.a = self.rrc_n(self.registers.a),

            // RL n
            0x10 => self.registers.b = self.rl_n(self.registers.b),
            0x11 => self.registers.c = self.rl_n(self.registers.c),
            0x12 => self.registers.d = self.rl_n(self.registers.d),
            0x13 => self.registers.e = self.rl_n(self.registers.e),
            0x14 => self.registers.h = self.rl_n(self.registers.h),
            0x15 => self.registers.l = self.rl_n(self.registers.l),
            0x16 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.rl_n(<D as MainDevice>::read(device, hl)?),
                )?
            }
            0x17 => self.registers.a = self.rl_n(self.registers.a),

            // RR n
            0x18 => self.registers.b = self.rr_n(self.registers.b),
            0x19 => self.registers.c = self.rr_n(self.registers.c),
            0x1a => self.registers.d = self.rr_n(self.registers.d),
            0x1b => self.registers.e = self.rr_n(self.registers.e),
            0x1c => self.registers.h = self.rr_n(self.registers.h),
            0x1d => self.registers.l = self.rr_n(self.registers.l),
            0x1e => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.rr_n(<D as MainDevice>::read(device, hl)?),
                )?
            }
            0x1f => self.registers.a = self.rr_n(self.registers.a),

            // SWAP n
            0x30 => self.registers.b = self.swap_n(self.registers.b),
            0x31 => self.registers.c = self.swap_n(self.registers.c),
            0x32 => self.registers.d = self.swap_n(self.registers.d),
            0x33 => self.registers.e = self.swap_n(self.registers.e),
            0x34 => self.registers.h = self.swap_n(self.registers.h),
            0x35 => self.registers.l = self.swap_n(self.registers.l),
            0x36 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.swap_n(<D as MainDevice>::read(device, hl)?),
                )?
            }
            0x37 => self.registers.a = self.swap_n(self.registers.a),

            // BIT 0,n
            0x40 => self.bit_b_n(0, self.registers.b),
            0x41 => self.bit_b_n(0, self.registers.c),
            0x42 => self.bit_b_n(0, self.registers.d),
            0x43 => self.bit_b_n(0, self.registers.e),
            0x44 => self.bit_b_n(0, self.registers.h),
            0x45 => self.bit_b_n(0, self.registers.l),
            0x46 => self.bit_b_n(0, <D as MainDevice>::read(device, self.registers.hl())?),
            0x47 => self.bit_b_n(0, self.registers.a),

            // BIT 1,n
            0x48 => self.bit_b_n(1, self.registers.b),
            0x49 => self.bit_b_n(1, self.registers.c),
            0x4a => self.bit_b_n(1, self.registers.d),
            0x4b => self.bit_b_n(1, self.registers.e),
            0x4c => self.bit_b_n(1, self.registers.h),
            0x4d => self.bit_b_n(1, self.registers.l),
            0x4e => self.bit_b_n(1, <D as MainDevice>::read(device, self.registers.hl())?),
            0x4f => self.bit_b_n(1, self.registers.a),

            // BIT 2,n
            0x50 => self.bit_b_n(2, self.registers.b),
            0x51 => self.bit_b_n(2, self.registers.c),
            0x52 => self.bit_b_n(2, self.registers.d),
            0x53 => self.bit_b_n(2, self.registers.e),
            0x54 => self.bit_b_n(2, self.registers.h),
            0x55 => self.bit_b_n(2, self.registers.l),
            0x56 => self.bit_b_n(2, <D as MainDevice>::read(device, self.registers.hl())?),
            0x57 => self.bit_b_n(2, self.registers.a),

            // BIT 3,n
            0x58 => self.bit_b_n(3, self.registers.b),
            0x59 => self.bit_b_n(3, self.registers.c),
            0x5a => self.bit_b_n(3, self.registers.d),
            0x5b => self.bit_b_n(3, self.registers.e),
            0x5c => self.bit_b_n(3, self.registers.h),
            0x5d => self.bit_b_n(3, self.registers.l),
            0x5e => self.bit_b_n(3, <D as MainDevice>::read(device, self.registers.hl())?),
            0x5f => self.bit_b_n(3, self.registers.a),

            // BIT 4,n
            0x60 => self.bit_b_n(4, self.registers.b),
            0x61 => self.bit_b_n(4, self.registers.c),
            0x62 => self.bit_b_n(4, self.registers.d),
            0x63 => self.bit_b_n(4, self.registers.e),
            0x64 => self.bit_b_n(4, self.registers.h),
            0x65 => self.bit_b_n(4, self.registers.l),
            0x66 => self.bit_b_n(4, <D as MainDevice>::read(device, self.registers.hl())?),
            0x67 => self.bit_b_n(4, self.registers.a),

            // BIT 5,n
            0x68 => self.bit_b_n(5, self.registers.b),
            0x69 => self.bit_b_n(5, self.registers.c),
            0x6a => self.bit_b_n(5, self.registers.d),
            0x6b => self.bit_b_n(5, self.registers.e),
            0x6c => self.bit_b_n(5, self.registers.h),
            0x6d => self.bit_b_n(5, self.registers.l),
            0x6e => self.bit_b_n(5, <D as MainDevice>::read(device, self.registers.hl())?),
            0x6f => self.bit_b_n(5, self.registers.a),

            // BIT 6,n
            0x70 => self.bit_b_n(6, self.registers.b),
            0x71 => self.bit_b_n(6, self.registers.c),
            0x72 => self.bit_b_n(6, self.registers.d),
            0x73 => self.bit_b_n(6, self.registers.e),
            0x74 => self.bit_b_n(6, self.registers.h),
            0x75 => self.bit_b_n(6, self.registers.l),
            0x76 => self.bit_b_n(6, <D as MainDevice>::read(device, self.registers.hl())?),
            0x77 => self.bit_b_n(6, self.registers.a),

            // BIT 7,n
            0x78 => self.bit_b_n(7, self.registers.b),
            0x79 => self.bit_b_n(7, self.registers.c),
            0x7a => self.bit_b_n(7, self.registers.d),
            0x7b => self.bit_b_n(7, self.registers.e),
            0x7c => self.bit_b_n(7, self.registers.h),
            0x7d => self.bit_b_n(7, self.registers.l),
            0x7e => self.bit_b_n(7, <D as MainDevice>::read(device, self.registers.hl())?),
            0x7f => self.bit_b_n(7, self.registers.a),

            // RES 0,n
            0x80 => self.registers.b = self.res_b_n(0, self.registers.b),
            0x81 => self.registers.c = self.res_b_n(0, self.registers.c),
            0x82 => self.registers.d = self.res_b_n(0, self.registers.d),
            0x83 => self.registers.e = self.res_b_n(0, self.registers.e),
            0x84 => self.registers.h = self.res_b_n(0, self.registers.h),
            0x85 => self.registers.l = self.res_b_n(0, self.registers.l),
            0x86 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.res_b_n(0, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0x87 => self.registers.a = self.res_b_n(0, self.registers.a),

            // RES 1,n
            0x88 => self.registers.b = self.res_b_n(1, self.registers.b),
            0x89 => self.registers.c = self.res_b_n(1, self.registers.c),
            0x8a => self.registers.d = self.res_b_n(1, self.registers.d),
            0x8b => self.registers.e = self.res_b_n(1, self.registers.e),
            0x8c => self.registers.h = self.res_b_n(1, self.registers.h),
            0x8d => self.registers.l = self.res_b_n(1, self.registers.l),
            0x8e => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.res_b_n(1, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0x8f => self.registers.a = self.res_b_n(1, self.registers.a),

            // RES 2,n
            0x90 => self.registers.b = self.res_b_n(2, self.registers.b),
            0x91 => self.registers.c = self.res_b_n(2, self.registers.c),
            0x92 => self.registers.d = self.res_b_n(2, self.registers.d),
            0x93 => self.registers.e = self.res_b_n(2, self.registers.e),
            0x94 => self.registers.h = self.res_b_n(2, self.registers.h),
            0x95 => self.registers.l = self.res_b_n(2, self.registers.l),
            0x96 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.res_b_n(2, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0x97 => self.registers.a = self.res_b_n(2, self.registers.a),

            // RES 3,n
            0x98 => self.registers.b = self.res_b_n(3, self.registers.b),
            0x99 => self.registers.c = self.res_b_n(3, self.registers.c),
            0x9a => self.registers.d = self.res_b_n(3, self.registers.d),
            0x9b => self.registers.e = self.res_b_n(3, self.registers.e),
            0x9c => self.registers.h = self.res_b_n(3, self.registers.h),
            0x9d => self.registers.l = self.res_b_n(3, self.registers.l),
            0x9e => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.res_b_n(3, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0x9f => self.registers.a = self.res_b_n(3, self.registers.a),

            // RES 4,n
            0xa0 => self.registers.b = self.res_b_n(4, self.registers.b),
            0xa1 => self.registers.c = self.res_b_n(4, self.registers.c),
            0xa2 => self.registers.d = self.res_b_n(4, self.registers.d),
            0xa3 => self.registers.e = self.res_b_n(4, self.registers.e),
            0xa4 => self.registers.h = self.res_b_n(4, self.registers.h),
            0xa5 => self.registers.l = self.res_b_n(4, self.registers.l),
            0xa6 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.res_b_n(4, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xa7 => self.registers.a = self.res_b_n(4, self.registers.a),

            // RES 5,n
            0xa8 => self.registers.b = self.res_b_n(5, self.registers.b),
            0xa9 => self.registers.c = self.res_b_n(5, self.registers.c),
            0xaa => self.registers.d = self.res_b_n(5, self.registers.d),
            0xab => self.registers.e = self.res_b_n(5, self.registers.e),
            0xac => self.registers.h = self.res_b_n(5, self.registers.h),
            0xad => self.registers.l = self.res_b_n(5, self.registers.l),
            0xae => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.res_b_n(5, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xaf => self.registers.a = self.res_b_n(5, self.registers.a),

            // RES 6,n
            0xb0 => self.registers.b = self.res_b_n(6, self.registers.b),
            0xb1 => self.registers.c = self.res_b_n(6, self.registers.c),
            0xb2 => self.registers.d = self.res_b_n(6, self.registers.d),
            0xb3 => self.registers.e = self.res_b_n(6, self.registers.e),
            0xb4 => self.registers.h = self.res_b_n(6, self.registers.h),
            0xb5 => self.registers.l = self.res_b_n(6, self.registers.l),
            0xb6 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.res_b_n(6, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xb7 => self.registers.a = self.res_b_n(6, self.registers.a),

            // RES 7,n
            0xb8 => self.registers.b = self.res_b_n(7, self.registers.b),
            0xb9 => self.registers.c = self.res_b_n(7, self.registers.c),
            0xba => self.registers.d = self.res_b_n(7, self.registers.d),
            0xbb => self.registers.e = self.res_b_n(7, self.registers.e),
            0xbc => self.registers.h = self.res_b_n(7, self.registers.h),
            0xbd => self.registers.l = self.res_b_n(7, self.registers.l),
            0xbe => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.res_b_n(7, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xbf => self.registers.a = self.res_b_n(7, self.registers.a),

            // SET 0,n
            0xc0 => self.registers.b = self.set_b_n(0, self.registers.b),
            0xc1 => self.registers.c = self.set_b_n(0, self.registers.c),
            0xc2 => self.registers.d = self.set_b_n(0, self.registers.d),
            0xc3 => self.registers.e = self.set_b_n(0, self.registers.e),
            0xc4 => self.registers.h = self.set_b_n(0, self.registers.h),
            0xc5 => self.registers.l = self.set_b_n(0, self.registers.l),
            0xc6 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.set_b_n(0, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xc7 => self.registers.a = self.set_b_n(0, self.registers.a),

            // SET 1,n
            0xc8 => self.registers.b = self.set_b_n(1, self.registers.b),
            0xc9 => self.registers.c = self.set_b_n(1, self.registers.c),
            0xca => self.registers.d = self.set_b_n(1, self.registers.d),
            0xcb => self.registers.e = self.set_b_n(1, self.registers.e),
            0xcc => self.registers.h = self.set_b_n(1, self.registers.h),
            0xcd => self.registers.l = self.set_b_n(1, self.registers.l),
            0xce => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.set_b_n(1, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xcf => self.registers.a = self.set_b_n(1, self.registers.a),

            // SET 2,n
            0xd0 => self.registers.b = self.set_b_n(2, self.registers.b),
            0xd1 => self.registers.c = self.set_b_n(2, self.registers.c),
            0xd2 => self.registers.d = self.set_b_n(2, self.registers.d),
            0xd3 => self.registers.e = self.set_b_n(2, self.registers.e),
            0xd4 => self.registers.h = self.set_b_n(2, self.registers.h),
            0xd5 => self.registers.l = self.set_b_n(2, self.registers.l),
            0xd6 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.set_b_n(2, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xd7 => self.registers.a = self.set_b_n(2, self.registers.a),

            // SET 3,n
            0xd8 => self.registers.b = self.set_b_n(3, self.registers.b),
            0xd9 => self.registers.c = self.set_b_n(3, self.registers.c),
            0xda => self.registers.d = self.set_b_n(3, self.registers.d),
            0xdb => self.registers.e = self.set_b_n(3, self.registers.e),
            0xdc => self.registers.h = self.set_b_n(3, self.registers.h),
            0xdd => self.registers.l = self.set_b_n(3, self.registers.l),
            0xde => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.set_b_n(3, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xdf => self.registers.a = self.set_b_n(3, self.registers.a),

            // SET 4,n
            0xe0 => self.registers.b = self.set_b_n(4, self.registers.b),
            0xe1 => self.registers.c = self.set_b_n(4, self.registers.c),
            0xe2 => self.registers.d = self.set_b_n(4, self.registers.d),
            0xe3 => self.registers.e = self.set_b_n(4, self.registers.e),
            0xe4 => self.registers.h = self.set_b_n(4, self.registers.h),
            0xe5 => self.registers.l = self.set_b_n(4, self.registers.l),
            0xe6 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.set_b_n(4, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xe7 => self.registers.a = self.set_b_n(4, self.registers.a),

            // SET 5,n
            0xe8 => self.registers.b = self.set_b_n(5, self.registers.b),
            0xe9 => self.registers.c = self.set_b_n(5, self.registers.c),
            0xea => self.registers.d = self.set_b_n(5, self.registers.d),
            0xeb => self.registers.e = self.set_b_n(5, self.registers.e),
            0xec => self.registers.h = self.set_b_n(5, self.registers.h),
            0xed => self.registers.l = self.set_b_n(5, self.registers.l),
            0xee => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.set_b_n(5, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xef => self.registers.a = self.set_b_n(5, self.registers.a),

            // SET 6,n
            0xf0 => self.registers.b = self.set_b_n(6, self.registers.b),
            0xf1 => self.registers.c = self.set_b_n(6, self.registers.c),
            0xf2 => self.registers.d = self.set_b_n(6, self.registers.d),
            0xf3 => self.registers.e = self.set_b_n(6, self.registers.e),
            0xf4 => self.registers.h = self.set_b_n(6, self.registers.h),
            0xf5 => self.registers.l = self.set_b_n(6, self.registers.l),
            0xf6 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.set_b_n(6, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xf7 => self.registers.a = self.set_b_n(6, self.registers.a),

            // SET 7,n
            0xf8 => self.registers.b = self.set_b_n(7, self.registers.b),
            0xf9 => self.registers.c = self.set_b_n(7, self.registers.c),
            0xfa => self.registers.d = self.set_b_n(7, self.registers.d),
            0xfb => self.registers.e = self.set_b_n(7, self.registers.e),
            0xfc => self.registers.h = self.set_b_n(7, self.registers.h),
            0xfd => self.registers.l = self.set_b_n(7, self.registers.l),
            0xfe => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.set_b_n(7, <D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0xff => self.registers.a = self.set_b_n(7, self.registers.a),

            // SRL n
            0x38 => self.registers.b = self.srl_n(self.registers.b),
            0x39 => self.registers.c = self.srl_n(self.registers.c),
            0x3a => self.registers.d = self.srl_n(self.registers.d),
            0x3b => self.registers.e = self.srl_n(self.registers.e),
            0x3c => self.registers.h = self.srl_n(self.registers.h),
            0x3d => self.registers.l = self.srl_n(self.registers.l),
            0x3e => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.srl_n(<D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0x3f => self.registers.a = self.srl_n(self.registers.a),

            // SLA n
            0x20 => self.registers.b = self.sla_n(self.registers.b),
            0x21 => self.registers.c = self.sla_n(self.registers.c),
            0x22 => self.registers.d = self.sla_n(self.registers.d),
            0x23 => self.registers.e = self.sla_n(self.registers.e),
            0x24 => self.registers.h = self.sla_n(self.registers.h),
            0x25 => self.registers.l = self.sla_n(self.registers.l),
            0x26 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.sla_n(<D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0x27 => self.registers.a = self.sla_n(self.registers.a),

            // SRA n
            0x28 => self.registers.b = self.sra_n(self.registers.b),
            0x29 => self.registers.c = self.sra_n(self.registers.c),
            0x2a => self.registers.d = self.sra_n(self.registers.d),
            0x2b => self.registers.e = self.sra_n(self.registers.e),
            0x2c => self.registers.h = self.sra_n(self.registers.h),
            0x2d => self.registers.l = self.sra_n(self.registers.l),
            0x2e => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.sra_n(<D as MainDevice>::read(device, self.registers.hl())?),
                )?
            }
            0x2f => self.registers.a = self.sra_n(self.registers.a),
        }

        Ok(cycles::prefixed(opcode))
    }

    fn exec_opcode<D: MainDevice>(&mut self, device: &mut D, opcode: u8) -> Result<u64, Error> {
        let mut branch = false;

        match opcode {
            // ADD A,n
            0x80 => self.add_n(self.registers.b),
            0x81 => self.add_n(self.registers.c),
            0x82 => self.add_n(self.registers.d),
            0x83 => self.add_n(self.registers.e),
            0x84 => self.add_n(self.registers.h),
            0x85 => self.add_n(self.registers.l),
            0x86 => self.add_n(<D as MainDevice>::read(device, self.registers.hl())?),
            0x87 => self.add_n(self.registers.a),
            0xc6 => {
                let d8 = self.fetch(device)?;
                self.add_n(d8)
            }
            // ADC A,n
            0x88 => self.adc_n(self.registers.b),
            0x89 => self.adc_n(self.registers.c),
            0x8a => self.adc_n(self.registers.d),
            0x8b => self.adc_n(self.registers.e),
            0x8c => self.adc_n(self.registers.h),
            0x8d => self.adc_n(self.registers.l),
            0x8e => self.adc_n(<D as MainDevice>::read(device, self.registers.hl())?),
            0x8f => self.adc_n(self.registers.a),
            0xce => {
                let d8 = self.fetch(device)?;
                self.adc_n(d8)
            }
            // SUB n
            0x90 => self.sub_n(self.registers.b),
            0x91 => self.sub_n(self.registers.c),
            0x92 => self.sub_n(self.registers.d),
            0x93 => self.sub_n(self.registers.e),
            0x94 => self.sub_n(self.registers.h),
            0x95 => self.sub_n(self.registers.l),
            0x96 => self.sub_n(<D as MainDevice>::read(device, self.registers.hl())?),
            0x97 => self.sub_n(self.registers.a),
            0xd6 => {
                let d8 = self.fetch(device)?;
                self.sub_n(d8)
            }
            // ADC A,n
            0x98 => self.sbc_n(self.registers.b),
            0x99 => self.sbc_n(self.registers.c),
            0x9a => self.sbc_n(self.registers.d),
            0x9b => self.sbc_n(self.registers.e),
            0x9c => self.sbc_n(self.registers.h),
            0x9d => self.sbc_n(self.registers.l),
            0x9e => self.sbc_n(<D as MainDevice>::read(device, self.registers.hl())?),
            0x9f => self.sbc_n(self.registers.a),
            0xde => {
                let d8 = self.fetch(device)?;
                self.sbc_n(d8)
            }
            // AND n
            0xa0 => self.and_n(self.registers.b),
            0xa1 => self.and_n(self.registers.c),
            0xa2 => self.and_n(self.registers.d),
            0xa3 => self.and_n(self.registers.e),
            0xa4 => self.and_n(self.registers.h),
            0xa5 => self.and_n(self.registers.l),
            0xa6 => self.and_n(<D as MainDevice>::read(device, self.registers.hl())?),
            0xa7 => self.and_n(self.registers.a),
            0xe6 => {
                let d8 = self.fetch(device)?;
                self.and_n(d8)
            }
            // XOR n
            0xa8 => self.xor_n(self.registers.b),
            0xa9 => self.xor_n(self.registers.c),
            0xaa => self.xor_n(self.registers.d),
            0xab => self.xor_n(self.registers.e),
            0xac => self.xor_n(self.registers.h),
            0xad => self.xor_n(self.registers.l),
            0xae => self.xor_n(<D as MainDevice>::read(device, self.registers.hl())?),
            0xaf => self.xor_n(self.registers.a),
            0xee => {
                let d8 = self.fetch(device)?;
                self.xor_n(d8)
            }
            // OR n
            0xb0 => self.or_n(self.registers.b),
            0xb1 => self.or_n(self.registers.c),
            0xb2 => self.or_n(self.registers.d),
            0xb3 => self.or_n(self.registers.e),
            0xb4 => self.or_n(self.registers.h),
            0xb5 => self.or_n(self.registers.l),
            0xb6 => self.or_n(<D as MainDevice>::read(device, self.registers.hl())?),
            0xb7 => self.or_n(self.registers.a),
            0xf6 => {
                let d8 = self.fetch(device)?;
                self.or_n(d8)
            }
            // CP n
            0xb8 => self.cp_n(self.registers.b),
            0xb9 => self.cp_n(self.registers.c),
            0xba => self.cp_n(self.registers.d),
            0xbb => self.cp_n(self.registers.e),
            0xbc => self.cp_n(self.registers.h),
            0xbd => self.cp_n(self.registers.l),
            0xbe => self.cp_n(<D as MainDevice>::read(device, self.registers.hl())?),
            0xbf => self.cp_n(self.registers.a),
            0xfe => {
                let d8 = self.fetch(device)?;
                self.cp_n(d8)
            }
            // INC n
            0x04 => self.registers.b = self.inc_n(self.registers.b),
            0x14 => self.registers.d = self.inc_n(self.registers.d),
            0x24 => self.registers.h = self.inc_n(self.registers.h),
            0x34 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.inc_n(<D as MainDevice>::read(device, hl)?),
                )?;
            }
            0x0c => self.registers.c = self.inc_n(self.registers.c),
            0x1c => self.registers.e = self.inc_n(self.registers.e),
            0x2c => self.registers.l = self.inc_n(self.registers.l),
            0x3c => self.registers.a = self.inc_n(self.registers.a),

            // DEC n
            0x05 => self.registers.b = self.dec_n(self.registers.b),
            0x15 => self.registers.d = self.dec_n(self.registers.d),
            0x25 => self.registers.h = self.dec_n(self.registers.h),
            0x35 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(
                    device,
                    hl,
                    self.dec_n(<D as MainDevice>::read(device, hl)?),
                )?;
            }
            0x0d => self.registers.c = self.dec_n(self.registers.c),
            0x1d => self.registers.e = self.dec_n(self.registers.e),
            0x2d => self.registers.l = self.dec_n(self.registers.l),
            0x3d => self.registers.a = self.dec_n(self.registers.a),

            // INC nn
            0x03 => {
                let r = self.inc_nn(self.registers.bc());
                self.registers.set_bc(r)
            }
            0x13 => {
                let r = self.inc_nn(self.registers.de());
                self.registers.set_de(r)
            }
            0x23 => {
                let r = self.inc_nn(self.registers.hl());
                self.registers.set_hl(r)
            }
            0x33 => self.registers.sp = self.inc_nn(self.registers.sp),
            // DEC nn
            0x0b => {
                let r = self.dec_nn(self.registers.bc());
                self.registers.set_bc(r)
            }
            0x1b => {
                let r = self.dec_nn(self.registers.de());
                self.registers.set_de(r)
            }
            0x2b => {
                let r = self.dec_nn(self.registers.hl());
                self.registers.set_hl(r)
            }
            0x3b => self.registers.sp = self.dec_nn(self.registers.sp),
            // ADD HL,nn
            0x09 => self.add_hl_nn(self.registers.bc()),
            0x19 => self.add_hl_nn(self.registers.de()),
            0x29 => self.add_hl_nn(self.registers.hl()),
            0x39 => self.add_hl_nn(self.registers.sp),

            // ADD SP,r8
            0xe8 => {
                let a = self.registers.sp;
                let b = i16::from(self.fetch_signed(device)?) as u16;
                flag!(self.registers, C = (a & 0xff) + (b & 0xff) > 0xff);
                flag!(self.registers, H = (a & 0xf) + (b & 0xf) > 0xf);
                flag!(self.registers, N = false);
                flag!(self.registers, Z = false);
                self.registers.sp = a.wrapping_add(b);
            }

            // SCF
            0x37 => {
                flag!(self.registers, N = false);
                flag!(self.registers, H = false);
                flag!(self.registers, C = true);
            }
            // CCF
            0x3f => {
                flag!(self.registers, N = false);
                flag!(self.registers, H = false);
                flag!(self.registers, C = !flag!(self.registers, C));
            }
            0x27 => {
                let mut a = self.registers.a;
                let mut adjust = if flag!(self.registers, C) { 0x60 } else { 0x00 };
                if flag!(self.registers, H) {
                    adjust |= 0x06;
                };
                if !flag!(self.registers, N) {
                    if a & 0x0f > 0x09 {
                        adjust |= 0x06;
                    };
                    if a > 0x99 {
                        adjust |= 0x60;
                    };
                    a = a.wrapping_add(adjust);
                } else {
                    a = a.wrapping_sub(adjust);
                }
                flag!(self.registers, C = adjust >= 0x60);
                flag!(self.registers, H = false);
                flag!(self.registers, Z = a == 0x00);
                self.registers.a = a;
            }
            // CPL
            0x2f => {
                self.registers.a = !self.registers.a;
                flag!(self.registers, N = true);
                flag!(self.registers, H = true);
            }

            // FIXME CONFLICT
            // according to https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
            // Z is reset, but according to http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
            // Z depends on the result
            //
            // UPDATE: opcode table is right
            0x07 => {
                self.registers.a = self.rlc_n(self.registers.a);
                flag!(self.registers, Z = false); // 09-op r,r.gb
            }
            0x17 => {
                self.registers.a = self.rl_n(self.registers.a);
                flag!(self.registers, Z = false);
            }
            0x0f => {
                self.registers.a = self.rrc_n(self.registers.a);
                flag!(self.registers, Z = false);
            }
            0x1f => {
                self.registers.a = self.rr_n(self.registers.a);
                flag!(self.registers, Z = false);
            }

            // LD B,n
            0x40 => self.registers.b = self.registers.b,
            0x41 => self.registers.b = self.registers.c,
            0x42 => self.registers.b = self.registers.d,
            0x43 => self.registers.b = self.registers.e,
            0x44 => self.registers.b = self.registers.h,
            0x45 => self.registers.b = self.registers.l,
            0x46 => self.registers.b = <D as MainDevice>::read(device, self.registers.hl())?,
            0x06 => self.registers.b = self.fetch(device)?,
            0x47 => self.registers.b = self.registers.a,
            // LD C,n
            0x48 => self.registers.c = self.registers.b,
            0x49 => self.registers.c = self.registers.c,
            0x4a => self.registers.c = self.registers.d,
            0x4b => self.registers.c = self.registers.e,
            0x4c => self.registers.c = self.registers.h,
            0x4d => self.registers.c = self.registers.l,
            0x4e => self.registers.c = <D as MainDevice>::read(device, self.registers.hl())?,
            0x0e => self.registers.c = self.fetch(device)?,
            0x4f => self.registers.c = self.registers.a,
            // LD D,n
            0x50 => self.registers.d = self.registers.b,
            0x51 => self.registers.d = self.registers.c,
            0x52 => self.registers.d = self.registers.d,
            0x53 => self.registers.d = self.registers.e,
            0x54 => self.registers.d = self.registers.h,
            0x55 => self.registers.d = self.registers.l,
            0x56 => self.registers.d = <D as MainDevice>::read(device, self.registers.hl())?,
            0x16 => self.registers.d = self.fetch(device)?,
            0x57 => self.registers.d = self.registers.a,
            // LD E,n
            0x58 => self.registers.e = self.registers.b,
            0x59 => self.registers.e = self.registers.c,
            0x5a => self.registers.e = self.registers.d,
            0x5b => self.registers.e = self.registers.e,
            0x5c => self.registers.e = self.registers.h,
            0x5d => self.registers.e = self.registers.l,
            0x5e => self.registers.e = <D as MainDevice>::read(device, self.registers.hl())?,
            0x1e => self.registers.e = self.fetch(device)?,
            0x5f => self.registers.e = self.registers.a,
            // LD H,n
            0x60 => self.registers.h = self.registers.b,
            0x61 => self.registers.h = self.registers.c,
            0x62 => self.registers.h = self.registers.d,
            0x63 => self.registers.h = self.registers.e,
            0x64 => self.registers.h = self.registers.h,
            0x65 => self.registers.h = self.registers.l,
            0x66 => self.registers.h = <D as MainDevice>::read(device, self.registers.hl())?,
            0x26 => self.registers.h = self.fetch(device)?,
            0x67 => self.registers.h = self.registers.a,
            // LD L,n
            0x68 => self.registers.l = self.registers.b,
            0x69 => self.registers.l = self.registers.c,
            0x6a => self.registers.l = self.registers.d,
            0x6b => self.registers.l = self.registers.e,
            0x6c => self.registers.l = self.registers.h,
            0x6d => self.registers.l = self.registers.l,
            0x6e => self.registers.l = <D as MainDevice>::read(device, self.registers.hl())?,
            0x2e => self.registers.l = self.fetch(device)?,
            0x6f => self.registers.l = self.registers.a,
            // LD (HL),n
            0x70 => <D as MainDevice>::write(device, self.registers.hl(), self.registers.b)?,
            0x71 => <D as MainDevice>::write(device, self.registers.hl(), self.registers.c)?,
            0x72 => <D as MainDevice>::write(device, self.registers.hl(), self.registers.d)?,
            0x73 => <D as MainDevice>::write(device, self.registers.hl(), self.registers.e)?,
            0x74 => <D as MainDevice>::write(device, self.registers.hl(), self.registers.h)?,
            0x75 => <D as MainDevice>::write(device, self.registers.hl(), self.registers.l)?,
            0x36 => <D as MainDevice>::write(device, self.registers.hl(), self.fetch(device)?)?,
            0x77 => <D as MainDevice>::write(device, self.registers.hl(), self.registers.a)?,
            // LD A,n
            0x78 => self.registers.a = self.registers.b,
            0x79 => self.registers.a = self.registers.c,
            0x7a => self.registers.a = self.registers.d,
            0x7b => self.registers.a = self.registers.e,
            0x7c => self.registers.a = self.registers.h,
            0x7d => self.registers.a = self.registers.l,
            0x7e => self.registers.a = <D as MainDevice>::read(device, self.registers.hl())?,
            0x3e => self.registers.a = self.fetch(device)?,
            0x7f => self.registers.a = self.registers.a,
            // LD (a16),SP
            0x08 => {
                let a16 = self.fetch_word(device)?;
                let sp = self.registers.sp;
                <D as MainDevice>::write(device, a16, (sp & 0xff) as u8)?;
                <D as MainDevice>::write(device, a16 + 1, ((sp >> 8) & 0xff) as u8)?;
            }
            // LD nn,d16
            0x01 => {
                let d = self.fetch_word(device)?;
                self.registers.set_bc(d)
            }
            0x11 => {
                let d = self.fetch_word(device)?;
                self.registers.set_de(d)
            }
            0x21 => {
                let d = self.fetch_word(device)?;
                self.registers.set_hl(d)
            }
            0x31 => self.registers.sp = self.fetch_word(device)?,
            // LD HL,SP+r8
            0xf8 => {
                let a = self.registers.sp;
                let b = i16::from(self.fetch_signed(device)?) as u16;
                flag!(self.registers, C = (a & 0x00ff) + (b & 0x00ff) > 0x00ff);
                flag!(self.registers, H = (a & 0x000f) + (b & 0x000f) > 0x000f);
                flag!(self.registers, N = false);
                flag!(self.registers, Z = false);
                self.registers.set_hl(a.wrapping_add(b));
            }
            // LD SP,HL
            0xf9 => self.registers.sp = self.registers.hl(),
            // LD (nn),A
            0x02 => <D as MainDevice>::write(device, self.registers.bc(), self.registers.a)?,
            0x12 => <D as MainDevice>::write(device, self.registers.de(), self.registers.a)?,
            0x22 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(device, hl, self.registers.a)?;
                self.registers.set_hl(hl.wrapping_add(1));
            }
            0x32 => {
                let hl = self.registers.hl();
                <D as MainDevice>::write(device, hl, self.registers.a)?;
                self.registers.set_hl(hl.wrapping_sub(1));
            }
            // LD A,(nn)
            0x0a => self.registers.a = <D as MainDevice>::read(device, self.registers.bc())?,
            0x1a => self.registers.a = <D as MainDevice>::read(device, self.registers.de())?,
            0x2a => {
                self.registers.a = {
                    let hl = self.registers.hl();
                    let d = <D as MainDevice>::read(device, hl)?;
                    self.registers.set_hl(hl.wrapping_add(1));
                    d
                }
            }
            0x3a => {
                self.registers.a = {
                    let hl = self.registers.hl();
                    let d = <D as MainDevice>::read(device, hl)?;
                    self.registers.set_hl(hl.wrapping_sub(1));
                    d
                }
            }

            0xe2 => <D as MainDevice>::write(
                device,
                0xff00 + u16::from(self.registers.c),
                self.registers.a,
            )?,
            0xf2 => {
                self.registers.a =
                    <D as MainDevice>::read(device, 0xff00 + u16::from(self.registers.c))?
            }

            0xe0 => {
                let a8 = self.fetch(device)? as u16;
                <D as MainDevice>::write(device, 0xff00 + a8, self.registers.a)?;
            }
            0xf0 => {
                let a8 = self.fetch(device)? as u16;
                self.registers.a = <D as MainDevice>::read(device, 0xff00 + a8)?;
            }

            0xea => <D as MainDevice>::write(device, self.fetch_word(device)?, self.registers.a)?,
            0xfa => {
                let a16 = self.fetch_word(device)?;
                self.registers.a = <D as MainDevice>::read(device, a16)?;
            }

            // POP nn
            0xc1 | 0xd1 | 0xe1 | 0xf1 => {
                let r = self.stack_pop(device)?;
                match opcode {
                    0xc1 => self.registers.set_bc(r),
                    0xd1 => self.registers.set_de(r),
                    0xe1 => self.registers.set_hl(r),
                    0xf1 => self.registers.set_af(r & 0xfff0),
                    // BUG
                    _ => panic!(),
                }
            }
            // PUSH nn
            0xc5 => self.stack_push(self.registers.bc(), device)?,
            0xd5 => self.stack_push(self.registers.de(), device)?,
            0xe5 => self.stack_push(self.registers.hl(), device)?,
            0xf5 => self.stack_push(self.registers.af(), device)?,

            // RET cc
            0xc0 => {
                if !flag!(self.registers, Z) {
                    self.registers.pc = self.stack_pop(device)?;

                    branch = true;
                }
            }
            0xd0 => {
                if !flag!(self.registers, C) {
                    self.registers.pc = self.stack_pop(device)?;

                    branch = true;
                }
            }
            0xc8 => {
                if flag!(self.registers, Z) {
                    self.registers.pc = self.stack_pop(device)?;

                    branch = true;
                }
            }
            0xd8 => {
                if flag!(self.registers, C) {
                    self.registers.pc = self.stack_pop(device)?;

                    branch = true;
                }
            }
            // RET
            0xc9 => self.registers.pc = self.stack_pop(device)?,
            // RETI
            0xd9 => {
                self.ime = true;
                self.registers.pc = self.stack_pop(device)?;
            }

            0xc7 => self.rst_n(0x00, device)?,
            0xd7 => self.rst_n(0x10, device)?,
            0xe7 => self.rst_n(0x20, device)?,
            0xf7 => self.rst_n(0x30, device)?,
            0xcf => self.rst_n(0x08, device)?,
            0xdf => self.rst_n(0x18, device)?,
            0xef => self.rst_n(0x28, device)?,
            0xff => self.rst_n(0x38, device)?,

            0xc4 => branch = self.call_c_n(!flag!(self.registers, Z), device)?,
            0xd4 => branch = self.call_c_n(!flag!(self.registers, C), device)?,
            0xcc => branch = self.call_c_n(flag!(self.registers, Z), device)?,
            0xdc => branch = self.call_c_n(flag!(self.registers, C), device)?,
            0xcd => self.call_n(device)?,

            0xc2 => branch = self.jp_c_n(!flag!(self.registers, Z), device)?,
            0xd2 => branch = self.jp_c_n(!flag!(self.registers, C), device)?,
            0xca => branch = self.jp_c_n(flag!(self.registers, Z), device)?,
            0xda => branch = self.jp_c_n(flag!(self.registers, C), device)?,
            0xc3 => self.registers.pc = self.fetch_word(device)?,
            0xe9 => {
                // The pdf was ambiguous. Verified with other emulators:
                // - https://github.com/taisel/GameBoy-Online/blob/master/js/GameBoyCore.js#L2086
                // - https://github.com/HFO4/gameboy.live/blob/master/gb/opcodes.go#L2103
                self.registers.pc = self.registers.hl()
            }

            0x20 => branch = self.jr_c(!flag!(self.registers, Z), device)?,
            0x30 => branch = self.jr_c(!flag!(self.registers, C), device)?,
            0x28 => branch = self.jr_c(flag!(self.registers, Z), device)?,
            0x38 => branch = self.jr_c(flag!(self.registers, C), device)?,
            0x18 => {
                self.jr_c(true, device)?;
            }

            // Misc/control instructions
            0x00 => {} // NOP
            //0x10 => unimplemented!("0x10 - STOP 0 - not implemented"), // STOP 0
            0x10 => {}
            0x76 => self.halt = true,
            0xf3 => self.ime = false,
            0xfb => self.ime = true,

            // CB opcode is handled by another method
            0xcb => unreachable!(),

            0xd3 | 0xdb | 0xdd | 0xe3 | 0xe4 | 0xeb..=0xed | 0xf4 | 0xfc | 0xfd => {
                panic!("Unknown opcode: {:#02x}", opcode)
            }
        }

        Ok(cycles::unprefixed(opcode, branch))
    }

    fn exec<D: MainDevice>(&mut self, device: &mut D) -> Result<u64, Error> {
        let opcode = self.fetch(device)?;
        if opcode == 0xcb {
            let opcode = self.fetch(device)?;
            self.exec_opcode_cb(device, opcode)
        } else {
            self.exec_opcode(device, opcode)
        }
    }

    fn fetch<D: MainDevice>(&mut self, device: &D) -> Result<u8, Error> {
        let opcode = <D as MainDevice>::read(device, self.registers.pc)?;
        self.registers.pc += 1;
        Ok(opcode)
    }

    fn fetch_word<D: MainDevice>(&mut self, device: &D) -> Result<u16, Error> {
        let lo = <D as MainDevice>::read(device, self.registers.pc)? as u16;
        let hi = <D as MainDevice>::read(device, self.registers.pc + 1)? as u16;
        self.registers.pc += 2;
        Ok((hi << 8) | lo)
    }

    fn fetch_signed<D: MainDevice>(&mut self, device: &D) -> Result<i8, Error> {
        let data: i8 = unsafe { std::mem::transmute(self.fetch(device)?) };
        Ok(data)
    }

    // Pushes word into the stack
    // Decrements SP by 2
    fn stack_push<D: MainDevice>(&mut self, nn: u16, device: &mut D) -> Result<(), Error> {
        self.registers.sp -= 2;
        device.write_word(self.registers.sp, nn)?;
        Ok(())
    }

    // Pops word from the stack
    // Increments SP by 2
    fn stack_pop<D: MainDevice>(&mut self, device: &D) -> Result<u16, Error> {
        let data = device.read_word(self.registers.sp)?;
        self.registers.sp += 2;
        Ok(data)
    }

    // Add n to A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 H C
    fn add_n(&mut self, n: u8) {
        let res = u16::from(self.registers.a) + u16::from(n);
        flag!(self.registers, Z = res.trailing_zeros() >= 8);
        flag!(self.registers, N = false);
        flag!(
            self.registers,
            H = (self.registers.a & 0xf) + (n & 0xf) > 0xf
        );
        flag!(self.registers, C = res > 0xff);
        self.registers.a = (res & 0xff) as u8;
    }

    // Add n + Carry flag to A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 H C
    fn adc_n(&mut self, n: u8) {
        let mut res = u16::from(self.registers.a) + u16::from(n);
        let carry = if flag!(self.registers, C) { 1 } else { 0 };
        res += u16::from(carry);

        flag!(self.registers, Z = res.trailing_zeros() >= 8);
        flag!(self.registers, N = false);
        flag!(
            self.registers,
            H = (self.registers.a & 0xf) + (n & 0xf) + carry > 0xf
        );
        flag!(self.registers, C = res > 0xff);
        self.registers.a = (res & 0xff) as u8;
    }

    // Subtract n from A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 1 H C
    fn sub_n(&mut self, n: u8) {
        let res = self.registers.a.wrapping_sub(n);
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = true);
        flag!(self.registers, H = n & 0xf > self.registers.a & 0xf);
        flag!(self.registers, C = n > self.registers.a);
        self.registers.a = res;
    }

    // Subtract n + Carry flag from A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 1 H C
    fn sbc_n(&mut self, n: u8) {
        let c = if flag!(self.registers, C) { 1 } else { 0 };
        let res = self.registers.a.wrapping_sub(n).wrapping_sub(c);
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = true);
        flag!(self.registers, H = (n & 0xf) + c > self.registers.a & 0xf);
        flag!(
            self.registers,
            C = u16::from(n) + u16::from(c) > u16::from(self.registers.a)
        );
        self.registers.a = res;
    }

    // Logically AND n with A, result in A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 1 0
    fn and_n(&mut self, n: u8) {
        let res = self.registers.a & n;
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = true);
        flag!(self.registers, C = false);
        self.registers.a = res;
    }

    // Logically OR n with A, result in A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 1 0
    fn or_n(&mut self, n: u8) {
        let res = self.registers.a | n;
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = false);
        flag!(self.registers, C = false);
        self.registers.a = res;
    }

    // Logically XOR n with A, result in A.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 0 0 0
    fn xor_n(&mut self, n: u8) {
        let res = self.registers.a ^ n;
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = false);
        flag!(self.registers, C = false);
        self.registers.a = res;
    }

    // Compare A with n.
    // n = A,B,C,D,E,H,(HL),#
    // Flags
    // Z 1 H C
    fn cp_n(&mut self, n: u8) {
        let a = self.registers.a;
        self.sub_n(n);
        self.registers.a = a;
    }

    // Increment register n.
    // n = A,B,C,D,E,H,(HL)
    // Flags
    // Z 0 H -
    fn inc_n(&mut self, n: u8) -> u8 {
        let res = n.wrapping_add(1);
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = n & 0xf == 0xf);
        res
    }

    // Decrement register n.
    // n = A,B,C,D,E,H,(HL)
    fn dec_n(&mut self, n: u8) -> u8 {
        let res = n.wrapping_sub(1);
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = true);
        flag!(self.registers, H = n.trailing_zeros() >= 4);
        res
    }

    // Add n to HL.
    // n = BC,DE,HL,SP
    // Flags
    // - 0 H C
    fn add_hl_nn(&mut self, nn: u16) {
        let res = u32::from(self.registers.hl()) + u32::from(nn);
        flag!(self.registers, N = false);
        flag!(
            self.registers,
            H = (self.registers.hl() & 0xfff) + (nn & 0xfff) > 0xfff
        );
        flag!(
            self.registers,
            C = u32::from(self.registers.hl()) + u32::from(nn) > 0xffff
        );
        self.registers.set_hl((res & 0xffff) as u16);
    }

    // Increment register nn.
    // n = BC,DE,HL,SP
    fn inc_nn(&mut self, nn: u16) -> u16 {
        nn.wrapping_add(1)
    }

    // Decrement register nn.
    // n = BC,DE,HL,SP
    fn dec_nn(&mut self, nn: u16) -> u16 {
        nn.wrapping_sub(1)
    }

    // Rotate n left. Old bit 7 to Carry flag
    // Flags:
    // Z 0 0 C
    fn rlc_n(&mut self, n: u8) -> u8 {
        let mut res = n << 1;
        if n & 0x80 != 0 {
            res |= 1;
        }
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = false);
        flag!(self.registers, C = n & 0x80 != 0);
        res
    }

    // Rotate n left through Carry flag.
    // Flags:
    // Z 0 0 C
    fn rl_n(&mut self, n: u8) -> u8 {
        let mut res = n << 1;
        if flag!(self.registers, C) {
            res |= 0x1;
        }
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = false);
        flag!(self.registers, C = n & 0x80 != 0);
        res
    }

    // Rotate n right. Old bit 0 to Carry flag
    // Flags:
    // Z 0 0 C
    fn rrc_n(&mut self, n: u8) -> u8 {
        let mut res = n >> 1;
        if n & 0x01 != 0 {
            res |= 0x80;
        }
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = false);
        flag!(self.registers, C = n & 0x1 != 0);
        res
    }

    // Rotate n right through Carry flag.
    // Flags:
    // Z 0 0 C
    fn rr_n(&mut self, n: u8) -> u8 {
        let mut res = n >> 1;
        if flag!(self.registers, C) {
            res |= 0x80;
        }
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = false);
        flag!(self.registers, C = n & 0x1 != 0);
        res
    }

    // Puts bit from register n into Z.
    // b = 0,1,2,3,4,5,6,7
    // n = A,B,C,D,E,H,L,(HL)
    // Flags:
    // Z 0 1 -
    fn bit_b_n(&mut self, b: u8, n: u8) {
        assert!(b <= 7);
        flag!(self.registers, Z = n & (1 << b) == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = true);
    }

    fn swap_n(&mut self, n: u8) -> u8 {
        let res = (n << 4) | (n >> 4);
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = false);
        flag!(self.registers, C = false);
        res
    }

    fn set_b_n(&mut self, b: u8, n: u8) -> u8 {
        n | (1 << b)
    }

    fn res_b_n(&mut self, b: u8, n: u8) -> u8 {
        n & !(1 << b)
    }

    // Shift n right into Carry. MSB set to 0.
    // n = A,B,C,D,E,H,L,(HL)
    // Flags
    // Z 0 0 C
    fn srl_n(&mut self, n: u8) -> u8 {
        let res = n >> 1;
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = false);
        flag!(self.registers, C = n & 0x1 != 0);
        res
    }

    // Shift n right into Carry. MSB set to 0.
    // n = A,B,C,D,E,H,L,(HL)
    // Flags
    // Z 0 0 C
    fn sra_n(&mut self, n: u8) -> u8 {
        let mut res = n >> 1;
        res |= n & 0x80;
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = false);
        flag!(self.registers, C = n & 0x1 != 0);
        res
    }

    // Shift n left into Carry. LSB of n set to 0
    fn sla_n(&mut self, n: u8) -> u8 {
        let res = n << 1;
        flag!(self.registers, Z = res == 0);
        flag!(self.registers, N = false);
        flag!(self.registers, H = false);
        flag!(self.registers, C = n & 0x80 != 0);
        res
    }

    // Pushes present address onto stack.
    // Jump to address $000 + n
    // n = 00,$08,$10,$18,$20,$28,$30,$38
    fn rst_n<D: MainDevice>(&mut self, n: u8, device: &mut D) -> Result<(), Error> {
        self.stack_push(self.registers.pc, device)?;
        self.registers.pc = n as u16;
        Ok(())
    }

    // Call Address n if following condition is true:
    // c = NZ, Call if Z flag is reset.
    // c = Z = Call if Z flag is set.
    // c = NC, Call if C flag is reset.
    // c = C = Call if C flag is set.
    fn call_c_n<D: MainDevice>(&mut self, branch: bool, device: &mut D) -> Result<bool, Error> {
        let n = self.fetch_word(device)?;
        if branch {
            self.stack_push(self.registers.pc, device)?;
            self.registers.pc = n;
        }
        Ok(branch)
    }

    // Push address of next instruction onto the stack and then jump to address n.
    fn call_n<D: MainDevice>(&mut self, device: &mut D) -> Result<(), Error> {
        let n = self.fetch_word(device)?;
        self.stack_push(self.registers.pc, device)?;
        self.registers.pc = n;
        Ok(())
    }

    // Jump to address n if following condition is true:
    // c = NZ, Call if Z flag is reset.
    // c = Z = Call if Z flag is set.
    // c = NC, Call if C flag is reset.
    // c = C =Call if C flag is set.
    fn jp_c_n<D: MainDevice>(&mut self, branch: bool, device: &mut D) -> Result<bool, Error> {
        let n = self.fetch_word(device)?;
        if branch {
            self.registers.pc = n;
        }
        Ok(branch)
    }

    // Add n to current address and jump tp it.
    fn jr_c<D: MainDevice>(&mut self, branch: bool, device: &mut D) -> Result<bool, Error> {
        let n = self.fetch_signed(device)?;
        if branch {
            let pc = i32::from(self.registers.pc) + i32::from(n);
            self.registers.pc = (pc & 0xffff) as u16;
        }
        Ok(branch)
    }
}

#[cfg(test)]
mod test {}
