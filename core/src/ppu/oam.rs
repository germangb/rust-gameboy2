use crate::dev::Device;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OAMTable {}

impl Device for OAMTable {
    fn debug_name() -> &'static str {
        "OAM Table"
    }

    fn read(&self, address: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, address: u16, data: u8) {
        todo!()
    }
}

#[cfg(test)]
mod test {}
