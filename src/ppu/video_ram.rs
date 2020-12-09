use crate::dev::Device;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VideoRam {}

impl Device for VideoRam {
    fn read(&self, address: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, address: u16, data: u8) {
        todo!()
    }
}

#[cfg(test)]
mod test {}
