use crate::device::{Device, Result};
use bitflags::bitflags;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Attributes: u8 {
        const PALETTE         = 0b00000111;
        const TILE_VRAM_BANK  = 0b00001000;
        const UNUSED          = 0b00010000;
        const HORIZONTAL_FLIP = 0b00100000;
        const VERTICAL_FLIP   = 0b01000000;
        const BG_OAM_PRIPRITY = 0b10000000;
    }
}

impl Attributes {
    pub fn palette(&self) -> u8 {
        self.bits & 0b111
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VideoRAM {
    // serde doesn't support big arrays so use a boxed slice instead of a boxed big array :(
    data: Box<[u8]>,
    bank: usize,
}

impl Default for VideoRAM {
    fn default() -> Self {
        Self {
            data: vec![0; 0x2000 * 2].into_boxed_slice(),
            bank: 0,
        }
    }
}

impl VideoRAM {
    fn bank_address(&self, address: u16) -> usize {
        self.bank * 0x2000 + (address as usize) - 0x8000
    }
}

impl Device for VideoRAM {
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
            address {
                0x8000..=0x9fff => Ok(self.data[self.bank_address(address)]),
                0xff4f => {
                    let bank = self.bank as u8;

                    // Reading from this register will return the number of the currently loaded
                    // VRAM bank in bit 0, and all other bits will be set to 1.
                    Ok(0xfe | bank)
                }
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
                0x8000..=0x9fff => self.data[self.bank_address(address)] = data,
                0xff4f => {
                    self.bank = (data & 1) as _;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Emulator};

    #[test]
    fn video_ram() {
        let mut emu = Emulator::new(NoCartridge);

        emu.write(0x8000, 1).unwrap();
        emu.write(0x9fff, 2).unwrap();

        assert_eq!(
            [1, 2],
            [emu.read(0x8000).unwrap(), emu.read(0x9fff).unwrap()]
        );
    }

    #[test]
    fn video_ram_bank() {
        todo!()
    }

    #[test]
    fn video_ram_attributes() {
        todo!()
    }
}
