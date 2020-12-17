use crate::{device::Device, error::Error};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct APU {}

impl Device for APU {
    const DEBUG_NAME: &'static str = "Audio Processing Unit";

    fn read(&self, address: u16) -> Result<u8, Error> {
        Ok(0xff)
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    #[ignore]
    fn apu() {
        todo!()
    }
}
