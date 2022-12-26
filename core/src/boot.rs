use crate::{
    device::Device,
    error::{ReadError, WriteError},
};
use log::info;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(all(feature = "boot", not(feature = "cgb")))]
const ROM: &[u8] = include_bytes!("../boot/boot.gb");
#[cfg(all(feature = "boot", feature = "cgb"))]
const ROM: &[u8] = include_bytes!("../boot/boot.gbc");

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Boot {
    enabled: bool,
}

impl Default for Boot {
    fn default() -> Self {
        Self {
            enabled: cfg!(feature = "boot"),
        }
    }
}

impl Boot {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Device for Boot {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        dev_read! {
            address {
                #[cfg(all(not(feature = "cgb"), feature = "boot"))]
                0x0000..=0x00ff if self.enabled => Ok(ROM[address as usize]),
                #[cfg(all(feature = "cgb", feature = "boot"))]
                0x0000..=0x00ff | 0x0150..=0x0900 if self.enabled => Ok(ROM[address as usize]),
                #[cfg(not(feature = "cgb"))]
                0x0000..=0x00ff => panic!("BOOT section disabled"),
                #[cfg(feature = "cgb")]
                0x0000..=0x00ff | 0x0150..=0x0900 => panic!("BOOT section disabled"),
                0xff50 => Ok(0x00),
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        dev_write! {
            address, data {
                #[cfg(not(feature = "cgb"))]
                0x0000..=0x00ff => panic!("BOOT section disabled"),
                #[cfg(feature = "cgb")]
                0x0000..=0x00ff | 0x0150..=0x0900 => panic!("BOOT section disabled"),
                0xff50 => {
                    if self.enabled && data != 0 {
                        info!("BOOT section disabled: {:#02x}", data);
                        self.enabled = false;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {}
