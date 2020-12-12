use byteorder::{ByteOrder, LittleEndian};
use log::error;

type Endianness = LittleEndian;

pub type Address = u16;
pub type Data = u8;

pub trait Device {
    const DEBUG_NAME: &'static str;

    fn read(&self, address: Address) -> u8;

    fn write(&mut self, address: Address, data: Data);

    fn read_word(&self, address: Address) -> u16 {
        let bytes = [self.read(address), self.read(address + 1)];
        Endianness::read_u16(&bytes[..])
    }

    fn write_word(&mut self, address: Address, data: u16) {
        let mut bytes = [0; 2];
        Endianness::write_u16(&mut &mut bytes[..], data);
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
