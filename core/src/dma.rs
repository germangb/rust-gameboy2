use crate::{
    device::{invalid_read, invalid_write, Device},
    EmulationStep, Update,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// OAM DMA transfer duration
const DURATION: u64 = 160;

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OamDma {
    dma: u8,
    clocks: u64,
}

impl OamDma {
    pub fn is_active(&self) -> bool {
        self.clocks != 0
    }

    pub fn start_address(&self) -> u16 {
        (self.dma as u16) << 8
    }
}

impl Update for OamDma {
    fn update(&mut self, step: &EmulationStep) {
        if self.clocks >= step.clock_ticks {
            self.clocks -= step.clock_ticks;
        } else {
            self.clocks = 0;
        }
    }
}

impl Device for OamDma {
    fn debug_name() -> &'static str {
        "OAM DMA Transfer"
    }

    fn read(&self, address: u16) -> u8 {
        if address != 0xff46 {
            invalid_read(address);
        }
        self.dma
    }

    fn write(&mut self, address: u16, data: u8) {
        if address != 0xff46 {
            invalid_write(address);
        }

        self.dma = data;
        self.clocks = DURATION;
    }
}

#[cfg(test)]
mod test {
    use super::OamDma;
    use crate::{device::Device, EmulationStep, Update};

    #[test]
    fn oam_dma_start_address() {
        let mut dma = OamDma::default();
        dma.write(0xff46, 0xab);

        assert_eq!(0xab00, dma.start_address());
    }

    #[test]
    fn oam_dma_time() {
        let mut dma = OamDma::default();
        let mut states = vec![dma.is_active()];

        dma.write(0xff46, 0xab);
        states.push(dma.is_active());

        for _ in 0..160 - 4 {
            dma.update(&EmulationStep { clock_ticks: 1 });
        }

        states.push(dma.is_active());
        dma.update(&EmulationStep { clock_ticks: 4 });
        states.push(dma.is_active());

        assert_eq!(vec![false, true, true, false], states);
    }
}
