use crate::device::{invalid_read, invalid_write, Device};
use bitflags::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Flags: u8 {
        const OBJ_TO_BG_PRIORITY = 0b_1_0000000;
        const Y_FLIP             = 0b0_1_000000;
        const X_FLIP             = 0b00_1_00000;
        const PAL_NUMBER         = 0b000_1_0000;
        const CGB_VRAM_BANK      = 0b0000_1_000;
        const CGB_PAL_0          = 0b00000_1_00;
        const CGB_PAL_1          = 0b000000_1_0;
        const CGB_PAL_2          = 0b0000000_1_;
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Entry {
    pub y: u8,
    pub x: u8,
    pub tile: u8,
    pub flags: Flags,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OAM {
    table: Box<[Entry]>,
}

impl Default for OAM {
    fn default() -> Self {
        Self {
            table: vec![Default::default(); 40].into_boxed_slice(),
        }
    }
}

impl OAM {
    pub fn table(&self) -> &[Entry] {
        &self.table[..]
    }
}

impl Device for OAM {
    const DEBUG_NAME: &'static str = "OAM Table";

    fn read(&self, address: u16) -> u8 {
        if let 0xfe00..=0xfe9f = address {
            let offset = address - 0xfe00;
            let index = offset as usize / 4;

            match offset % 4 {
                0 => self.table[index].y,
                1 => self.table[index].x,
                2 => self.table[index].tile,
                3 => self.table[index].flags.bits,
                _ => unreachable!(),
            }
        } else {
            invalid_read(address)
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        if let 0xfe00..=0xfe9f = address {
            let offset = address - 0xfe00;
            let index = offset as usize / 4;

            match offset % 4 {
                0 => self.table[index].y = data,
                1 => self.table[index].x = data,
                2 => self.table[index].tile = data,
                3 => self.table[index].flags = Flags::from_bits(data).unwrap(),
                _ => unreachable!(),
            }
        } else {
            invalid_write(address)
        }
    }
}

#[cfg(test)]
mod test {
    use super::Entry;
    use crate::{cartridge::NoCartridge, device::Device, ppu::oam::Flags, Emulator};

    #[test]
    fn oam() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xfe00, 0x12);
        emu.write(0xfe9f, 0xef);

        assert_eq!(
            [0x12, 0x12, 0x12, 0xef, 0xef, 0xef,],
            [
                emu.read(0xfe00),
                emu.ppu.read(0xfe00),
                emu.ppu.oam.read(0xfe00),
                emu.read(0xfe9f),
                emu.ppu.read(0xfe9f),
                emu.ppu.oam.read(0xfe9f),
            ]
        );
    }

    #[test]
    fn oam_flags() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xfe03, 0b10101010);
        emu.write(0xfe06, 0b10101010);
        emu.write(0xfe09, 0b10101010);

        assert_eq!(
            [0b10101010, 0b10101010, 0b10101010,],
            [emu.read(0xfe03), emu.read(0xfe06), emu.read(0xfe09),]
        );
        assert_eq!(
            [
                Flags::from_bits(0b10101010).unwrap(),
                Flags::from_bits(0b10101010).unwrap(),
                Flags::from_bits(0b10101010).unwrap(),
            ],
            [
                emu.ppu.oam.table[0].flags,
                emu.ppu.oam.table[0].flags,
                emu.ppu.oam.table[0].flags,
            ]
        );
    }

    #[test]
    fn entry_size() {
        assert_eq!(4, std::mem::size_of::<Entry>())
    }
}
