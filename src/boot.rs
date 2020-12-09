use crate::dev::{invalid_read, invalid_write, Device};
use log::info;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const ROM: &[u8] = include_bytes!("boot/boot.gb");

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Boot {
    enabled: bool,
}

impl Boot {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Device for Boot {
    fn debug_name() -> Option<&'static str> {
        Some("Boot")
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x00ff if self.enabled => ROM[address as usize],
            0xff50 => {
                if self.enabled {
                    0x00
                } else {
                    0xff
                }
            }
            _ => invalid_read(address),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x00ff => unreachable!(),
            0xff50 => {
                if self.enabled && data != 0 {
                    info!(
                        "BOOT section disabled by write of non-zero ({:#02x}) to {:#04x}",
                        data, address
                    );

                    self.enabled = false;
                }
            }
            _ => invalid_write(address),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn boot() {}
}
