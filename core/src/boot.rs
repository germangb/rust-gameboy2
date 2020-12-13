use crate::device::{invalid_read, invalid_write, Device};
use log::{error, info};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "boot")]
const ROM: &[u8] = include_bytes!("boot/dmg_boot.bin");

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Boot {
    enabled: bool,
}

impl Default for Boot {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Boot {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Device for Boot {
    const DEBUG_NAME: &'static str = "BOOT Section";

    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x00ff if self.enabled => {
                if cfg!(feature = "boot") {
                    ROM[address as usize]
                } else {
                    error!("Emulator must be build with the \"boot\" feature flag in order to run the boot sequence");
                    panic!();
                }
            }
            0x0000..=0x00ff => panic!("BOOT section disabled"),
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
            0x0000..=0x00ff => panic!("BOOT section disabled"),
            0xff50 => {
                if self.enabled && data != 0 {
                    info!("BOOT section disabled: {:#02x}", data);

                    self.enabled = false;
                }
            }
            _ => invalid_write(address),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        boot::{Boot, ROM},
        device::Device,
    };

    #[cfg(feature = "boot")]
    #[test]
    fn boot() {
        let mut boot = Boot::default();

        let mut read = Vec::new();
        for i in 0..=0xff {
            read.push(boot.read(i));
        }

        assert_eq!(ROM, &read[..])
    }

    #[test]
    fn boot_is_enabled() {
        let mut boot = Boot::default();

        let e0 = boot.is_enabled();
        boot.write(0xff50, 0x1);
        let e1 = boot.is_enabled();

        assert_eq!([true, false], [e0, e1]);
    }

    #[test]
    #[should_panic]
    fn boot_panic_read() {
        let mut boot = Boot::default();

        boot.write(0xff50, 1);
        boot.read(0x00);
    }

    #[test]
    #[should_panic]
    fn boot_panic_write() {
        let mut boot = Boot::default();

        boot.write(0xff50, 1);
        boot.write(0xff, 0);
    }
}
