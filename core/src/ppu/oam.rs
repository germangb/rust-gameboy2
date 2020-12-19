use crate::{device::Device, error::Error};
use bitflags::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Flags: u8 {
        const OBJ_TO_BG_PRIORITY = 0b10000000;
        const Y_FLIP             = 0b01000000;
        const X_FLIP             = 0b00100000;
        const PAL_NUMBER         = 0b00010000;
        const CGB_VRAM_BANK      = 0b00001000;
        const CGB_PAL_NUMBER     = 0b00000111;
    }
}

impl Flags {
    pub fn palette(&self) -> usize {
        (self.bits & 0b111) as _
    }

    pub fn bank(&self) -> usize {
        if self.contains(Flags::CGB_VRAM_BANK) {
            1
        } else {
            0
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Entry {
    pub y: u8,
    pub x: u8,
    pub tile_index: u8,
    pub flags: Flags,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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
    fn read(&self, address: u16) -> Result<u8, Error> {
        if let 0xfe00..=0xfe9f = address {
            let offset = address - 0xfe00;
            let index = offset as usize / 4;

            let data = match offset % 4 {
                0 => self.table[index].y,
                1 => self.table[index].x,
                2 => self.table[index].tile_index,
                3 => self.table[index].flags.bits,
                _ => unreachable!(),
            };

            Ok(data)
        } else {
            Err(Error::InvalidAddr(address))
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), Error> {
        if let 0xfe00..=0xfe9f = address {
            let offset = address - 0xfe00;
            let index = offset as usize / 4;

            match offset % 4 {
                0 => self.table[index].y = data,
                1 => self.table[index].x = data,
                2 => self.table[index].tile_index = data,
                3 => self.table[index].flags = Flags::from_bits(data).unwrap(),
                _ => unreachable!(),
            }

            Ok(())
        } else {
            Err(Error::InvalidAddr(address))
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

        emu.write(0xfe00, 0x12).unwrap();
        emu.write(0xfe9f, 0xef).unwrap();

        assert_eq!(
            [0x12, 0x12, 0x12, 0xef, 0xef, 0xef,],
            [
                emu.read(0xfe00).unwrap(),
                emu.ppu.read(0xfe00).unwrap(),
                emu.ppu.oam.read(0xfe00).unwrap(),
                emu.read(0xfe9f).unwrap(),
                emu.ppu.read(0xfe9f).unwrap(),
                emu.ppu.oam.read(0xfe9f).unwrap(),
            ]
        );
    }

    #[test]
    fn oam_flags() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0xfe03, 0b10101010).unwrap();
        emu.write(0xfe06, 0b10101010).unwrap();
        emu.write(0xfe09, 0b10101010).unwrap();

        assert_eq!(
            [0b10101010, 0b10101010, 0b10101010,],
            [
                emu.read(0xfe03).unwrap(),
                emu.read(0xfe06).unwrap(),
                emu.read(0xfe09).unwrap(),
            ]
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
