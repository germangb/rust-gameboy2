use byteorder::{ByteOrder, LittleEndian};
use educe::Educe;
use log::{error, info};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type Address = u16;

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

/// Adds logging (using the log crate) to trace reads and writes.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Educe)]
#[educe(Deref, DerefMut)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct LogDevice<D>(#[educe(Deref, DerefMut)] D);

impl<D: Device> Device for LogDevice<D> {
    fn debug_name() -> Option<&'static str> {
        D::debug_name()
    }

    fn read(&self, address: u16) -> u8 {
        read(self, address)
    }

    fn write(&mut self, address: u16, data: u8) {
        write(self, address, data)
    }
}

/// Read from device.
///
/// Unlike when using the trait method directly, this method will use the log
/// crate to add info logs.
pub fn read<D: Device>(device: &D, address: Address) -> u8 {
    info!(
        "Reading from device (name = {:?}): {:#04x}",
        D::debug_name(),
        address
    );
    let data = device.read(address);
    info!("Read data: {:#02x} ({})", data, data);
    data
}

/// Write data to device
///
/// Unlike when using the trait method directly, this method will use the log
/// crate to add info logs.
pub fn write<D: Device>(device: &mut D, address: Address, data: u8) {
    #[rustfmt::skip]
    info!("Writing {:#02x} to device (name = {:?}): {:#04x}", data, D::debug_name(), address);
    device.write(address, data);
}

/// Memory-mapped device you can read & write bytes from and to.
pub trait Device {
    fn debug_name() -> Option<&'static str> {
        None
    }

    /// Read byte
    fn read(&self, address: Address) -> u8;

    /// Write byte.
    fn write(&mut self, address: Address, data: u8);

    /// Read a word.
    fn read_word(&self, address: Address) -> u16 {
        let bytes = [self.read(address), self.read(address + 1)];
        LittleEndian::read_u16(&bytes[..])
    }

    /// Write a word.
    fn write_word(&mut self, address: Address, data: u16) {
        let mut bytes = [0; 2];
        LittleEndian::write_u16(&mut &mut bytes[..], data);
        self.write(address, bytes[0]);
        self.write(address + 1, bytes[1]);
    }
}

#[cfg(test)]
mod test {
    use super::Device;

    type TestDevice = Box<[u8; 0x10000]>;

    impl Device for TestDevice {
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
