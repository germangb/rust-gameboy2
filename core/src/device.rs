use byteorder::{ByteOrder, LittleEndian};
use educe::Educe;
use log::{error, info};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type Address = u16;
pub type Data = u8;

pub trait Device {
    const DEBUG_NAME: &'static str;

    fn read(&self, address: Address) -> u8;

    fn write(&mut self, address: Address, data: Data);

    fn read_word(&self, address: Address) -> u16 {
        let bytes = [self.read(address), self.read(address + 1)];
        LittleEndian::read_u16(&bytes[..])
    }

    fn write_word(&mut self, address: Address, data: u16) {
        let mut bytes = [0; 2];
        LittleEndian::write_u16(&mut &mut bytes[..], data);
        self.write(address, bytes[0]);
        self.write(address + 1, bytes[1]);
    }
}

// returned for invalid memory accesses
// panics on release builds
pub(crate) fn invalid_read(address: Address) -> u8 {
    error!("Read from invalid address: {:#04x}", address);
    #[cfg(debug_assertions)]
    panic!();
    #[cfg(not(debug_assertions))]
    0
}

// returned for invalid memory accesses
// panics on release builds
pub(crate) fn invalid_write(address: Address) {
    error!("Write to invalid address: {:#04x}", address);
    #[cfg(debug_assertions)]
    panic!();
}

// Adds logging (using the log crate) to trace reads and writes.
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Educe)]
#[educe(Deref, DerefMut)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct LogDevice<D>(#[educe(Deref, DerefMut)] pub D);

impl<D: Device> Device for LogDevice<D> {
    const DEBUG_NAME: &'static str = D::DEBUG_NAME;

    fn read(&self, address: u16) -> u8 {
        info!("Read from device ({}): {:#04x}", D::DEBUG_NAME, address);
        let data = self.0.read(address);
        info!("Data: {:#02x} ({})", data, data);
        data
    }

    fn write(&mut self, address: u16, data: u8) {
        info!(
            "Write {:#02x} to device ({}): {:#04x}",
            data,
            D::DEBUG_NAME,
            address
        );
        self.0.write(address, data);
    }
}

#[cfg(test)]
mod test {
    use super::Device;

    type TestDevice = Box<[u8; 0x10000]>;

    impl Device for TestDevice {
        const DEBUG_NAME: &'static str = "Test";

        fn read(&self, address: u16) -> u8 {
            self[address as usize]
        }

        fn write(&mut self, address: u16, data: u8) {
            self[address as usize] = data
        }
    }

    fn test_device() -> TestDevice {
        Box::new([0; 0x10000])
    }

    #[test]
    fn read_write_word() {
        let mut device = test_device();

        device.write_word(0x0000, 0x1234);
        device.write_word(0x0100, 0xabcd);

        assert_eq!(0x1234, device.read_word(0x0000));
        assert_eq!(0xabcd, device.read_word(0x0100));
    }
}
