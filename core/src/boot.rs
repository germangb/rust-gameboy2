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
        Self { enabled: true }
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
                #[cfg(not(feature = "cgb"))]
                0x0000..=0x00ff if self.enabled => {
                    if cfg!(feature = "boot") {
                        Ok(ROM[address as usize])
                    } else {
                        panic!("Emulator must be build with the \"boot\" feature flag in order to run the boot sequence");
                    }
                }
                #[cfg(feature = "cgb")]
                0x0000..=0x00ff | 0x0150..=0x0900 if self.enabled => {
                    if cfg!(feature = "boot") {
                        Ok(ROM[address as usize])
                    } else {
                        panic!("Emulator must be build with the \"boot\" feature flag in order to run the boot sequence");
                    }
                }
                #[cfg(not(feature = "cgb"))]
                0x0000..=0x00ff => panic!("BOOT section disabled"),
                #[cfg(feature = "cgb")]
                0x0000..=0x00ff | 0x0150..=0x0900 => panic!("BOOT section disabled"),
                0xff50 => if self.enabled { Ok(0x00) } else { Ok(0xff) },
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
mod test {
    use crate::{
        boot::{Boot, ROM},
        device::Device,
    };

    #[cfg(feature = "boot")]
    #[test]
    fn boot() {
        let boot = Boot::default();

        let mut read = Vec::new();
        for i in 0..=0xff {
            read.push(boot.read(i));
        }

        assert_eq!(
            Ok(ROM.to_vec()),
            read.into_iter().collect::<Result<Vec<u8>, _>>()
        )
    }

    #[test]
    fn boot_is_enabled() {
        let mut boot = Boot::default();

        let e0 = boot.is_enabled();
        boot.write(0xff50, 0x1).unwrap();
        let e1 = boot.is_enabled();

        assert_eq!([true, false], [e0, e1]);
    }

    #[test]
    #[should_panic]
    fn boot_panic_read() {
        let mut boot = Boot::default();

        boot.write(0xff50, 1).unwrap();
        boot.read(0x0000).unwrap();
    }

    #[test]
    #[should_panic]
    fn boot_panic_write() {
        let mut boot = Boot::default();

        boot.write(0xff50, 1).unwrap();
        boot.write(0x00ff, 0).unwrap();
    }
}
