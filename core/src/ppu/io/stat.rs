use crate::{
    device::{invalid_read, invalid_write, Device},
    EmulationStep, Update,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct STAT {
    stat: u8,
}

impl STAT {
    pub fn lyc_ly_int(&self) -> bool {
        todo!()
    }

    pub fn oam_int(&self) -> bool {
        todo!()
    }

    pub fn vblank_int(&self) -> bool {
        todo!()
    }

    pub fn hblank_int(&self) -> bool {
        todo!()
    }

    pub fn coincidence_flag(&self) -> bool {
        todo!()
    }

    pub fn mode(&self) -> u8 {
        todo!()
    }
}

impl Update for STAT {
    fn update(&mut self, step: &EmulationStep) {
        todo!()
    }
}

impl Device for STAT {
    fn debug_name() -> &'static str {
        "STAT"
    }

    fn read(&self, address: u16) -> u8 {
        if address != 0xff41 {
            invalid_read(address);
        }

        self.stat
    }

    fn write(&mut self, address: u16, data: u8) {
        if address != 0xff41 {
            invalid_write(address);
        }

        self.stat &= 0b0000_0111;
        self.stat |= data & 0b1111_1000
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn stat() {
        todo!()
    }
}
