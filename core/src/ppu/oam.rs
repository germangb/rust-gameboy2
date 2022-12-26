use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
use bitflags::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::slice;

bitflags! {
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Flags: u8 {
        const OBJ_TO_BG_PRIORITY = 0b1000_0000;
        const Y_FLIP             = 0b0100_0000;
        const X_FLIP             = 0b0010_0000;
        const PAL_NUMBER         = 0b0001_0000;
        const CGB_VRAM_BANK      = 0b0000_1000;
        const CGB_PAL_NUMBER     = 0b0000_0111;
    }
}

impl Flags {
    #[cfg(feature = "cgb")]
    pub fn palette(&self) -> usize {
        (self.bits & 0b111) as _
    }

    #[cfg(feature = "cgb")]
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
    pub index: u8,
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
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xfe00..=0xfe9f => {
                    let off = (address - 0xfe00) as usize;
                    match off % 4 {
                        0 => Ok(self.table[off / 4].y),
                        1 => Ok(self.table[off / 4].x),
                        2 => Ok(self.table[off / 4].index),
                        3 => Ok(self.table[off / 4].flags.bits),
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xfe00..=0xfe9f => {
                    let off = (address - 0xfe00) as usize;
                    match off % 4 {
                        0 => self.table[off / 4].y = data,
                        1 => self.table[off / 4].x = data,
                        2 => self.table[off / 4].index = data,
                        3 => self.table[off / 4].flags = Flags::from_bits(data).unwrap(),
                        _ => unreachable!(),
                    }
                }
            }
        }
        Ok(())
    }

    fn write_exact(&mut self, address: u16, buf: &[u8]) -> Result<(), WriteError> {
        match address {
            0xfe00 if buf.len() == 0xa0 => {
                let oam_raw_ptr = self.table.as_mut_ptr() as *mut u8;
                let oam = unsafe { slice::from_raw_parts_mut(oam_raw_ptr, 0xa0) };
                oam.copy_from_slice(buf);
                Ok(())
            }
            _ => self.write_exact_fallback(address, buf),
        }
    }
}

#[cfg(test)]
mod test {}
