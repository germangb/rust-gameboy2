use crate::{
    device::Device,
    error::{Component, ReadError, WriteError},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct APU {}

impl Device for APU {
    #[allow(unused_variables)]
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        Err(ReadError::AddrNotImpl(address, Some(Component::APU)))
    }

    #[allow(unused_variables)]
    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        Err(WriteError::AddrNotImpl(address, data, Some(Component::APU)))
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
