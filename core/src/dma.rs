use crate::{
    device::{Device, Result},
    irq, Update,
};
use log::info;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// OAM DMA transfer duration
const DURATION: u64 = 160;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DMA {
    dma: u8,
    clocks: u64,
}

impl DMA {
    pub fn is_active(&self) -> bool {
        self.clocks != 0
    }

    pub fn start_address(&self) -> u16 {
        (self.dma as u16) << 8
    }
}

impl Update for DMA {
    fn update(&mut self, ticks: u64, _: &mut irq::Flags) {
        if self.clocks >= ticks {
            self.clocks -= ticks;
        } else {
            self.clocks = 0;
        }
    }
}

impl Device for DMA {
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
            address {
                0xff46 => Ok(self.dma),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
                0xff46 => {
                    info!("OAM DMA Register: {:02x}", self.dma);

                    self.dma = data;
                    self.clocks = DURATION;
                }
            }
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HDMA {
    pub hdma1: u8,
    pub hdma2: u8,
    pub hdma3: u8,
    pub hdma4: u8,
}

impl Device for HDMA {
    fn read(&self, address: u16) -> Result<u8> {
        device_match! {
            address {
                0xff51..=0xff54 => Ok(0xff),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<()> {
        device_match! {
            address {
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
    use super::DMA;
    use crate::{cartridge::NoCartridge, device::Device, Emulator, Update};

    #[test]
    fn oam_dma_start_address() {
        let mut emu = Emulator::new(NoCartridge);
        emu.write(0xff46, 0xab).unwrap();

        assert_eq!(0xab00, emu.dma.start_address());
    }

    #[test]
    fn oam_dma_time() {
        let mut emu = Emulator::new(NoCartridge);
        let mut states = vec![emu.dma.is_active()];

        emu.write(0xff46, 0xab).unwrap();
        states.push(emu.dma.is_active());

        for _ in 0..160 - 4 {
            emu.dma.update(1, &mut Default::default());
        }

        states.push(emu.dma.is_active());
        emu.dma.update(4, &mut Default::default());
        states.push(emu.dma.is_active());

        assert_eq!(vec![false, true, true, false], states);
    }
}
