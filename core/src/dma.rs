use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OAMDMA {
    pub dma: u8,
}

impl Device for OAMDMA {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff46 => Ok(self.dma),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff46 => self.dma = data,
            }
        }

        Ok(())
    }
}

#[cfg(feature = "cgb")]
#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VRAMDMA {
    pub hdma1: u8,
    pub hdma2: u8,
    pub hdma3: u8,
    pub hdma4: u8,
}

#[cfg(feature = "cgb")]
impl Device for VRAMDMA {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                0xff51..=0xff54 => Ok(0xff),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                0xff51 => self.hdma1 = data,
                0xff52 => self.hdma2 = data,
                0xff53 => self.hdma3 = data,
                0xff54 => self.hdma4 = data,
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cartridge::NoCartridge, device::Device, Update, LR35902};

    #[test]
    fn oam_dma_start_address() {
        let mut emu = LR35902::new(NoCartridge);
        emu.write(0xff46, 0xab).unwrap();

        assert_eq!(0xab00, emu.oam_dma.start_address());
    }

    #[test]
    fn oam_dma_time() {
        let mut emu = LR35902::new(NoCartridge);
        let mut states = vec![emu.oam_dma.is_active()];

        emu.write(0xff46, 0xab).unwrap();
        states.push(emu.oam_dma.is_active());

        for _ in 0..160 - 4 {
            emu.oam_dma.update(1, &mut Default::default());
        }

        states.push(emu.oam_dma.is_active());
        emu.oam_dma.update(4, &mut Default::default());
        states.push(emu.oam_dma.is_active());

        assert_eq!(vec![false, true, true, false], states);
    }
}
