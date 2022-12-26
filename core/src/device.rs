use crate::error::{Component, ReadError, WriteError};
use byteorder::{ByteOrder, LittleEndian};

pub trait Device {
    /// Read byte from given address.
    /// May return Err if `address` is not mapped to the device.
    fn read(&self, address: u16) -> Result<u8, ReadError>;

    /// Write a byte to given address.
    /// May return Err if `address` is not mapped to the device.
    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError>;

    /// Read a exact amount of bytes, starting at the given address, into the
    /// given output buffer. Device implementors may re-implement this method
    /// for speed.
    fn read_exact(&self, address: u16, buf: &mut [u8]) -> Result<(), ReadError> {
        self.read_exact_fallback(address, buf)
    }

    /// Fallback method for read_exact.
    fn read_exact_fallback(&self, address: u16, buf: &mut [u8]) -> Result<(), ReadError> {
        for (i, out) in buf.iter_mut().enumerate() {
            *out = self.read(address + (i as u16))?;
        }
        Ok(())
    }

    /// Write the exact amount of bytes in the slice, starting at the given
    /// address. Device implementors may re-define this method for speed.
    fn write_exact(&mut self, address: u16, buf: &[u8]) -> Result<(), WriteError> {
        self.write_exact_fallback(address, buf)
    }

    /// Fallback method for write_exact.
    fn write_exact_fallback(&mut self, address: u16, buf: &[u8]) -> Result<(), WriteError> {
        for (i, data) in buf.iter().enumerate() {
            self.write(address + i as u16, *data)?;
        }
        Ok(())
    }

    /// Return the name of the device at the given address.
    fn name(&self, address: u16) -> Option<&str> {
        None
    }
}

pub trait MemoryBus: Device {
    fn read(&self, address: u16) -> Result<u8, ReadError> {
        match <Self as Device>::read(self, address) {
            Ok(b) => Ok(b),
            Err(err) => {
                match err {
                    ReadError::UnknownAddr(_) => log::warn!("{err}"),
                    ReadError::AddrNotImpl(_, Some(Component::APU)) => {}
                    ReadError::AddrNotImpl(_, Some(Component::Serial)) => {}
                    _ => log::error!("{err}"),
                }
                Ok(0x00)
            }
        }
    }

    fn write(&mut self, address: u16, data: u8) -> Result<(), WriteError> {
        match <Self as Device>::write(self, address, data) {
            Ok(_) => Ok(()),
            Err(err) => {
                match err {
                    WriteError::UnknownAddr(_, _) => log::warn!("{err}"),
                    WriteError::AddrNotImpl(_, _, Some(Component::APU)) => {}
                    WriteError::AddrNotImpl(_, _, Some(Component::Serial)) => {}
                    _ => log::error!("{err}"),
                }
                Ok(())
            }
        }
    }

    /// Read little-endian u16 word from given address.
    /// May return Err if `address` or `address + 1` are not mapped to the
    /// device.
    fn read_word(&self, address: u16) -> Result<u16, ReadError> {
        let bytes = [
            <Self as MemoryBus>::read(self, address)?,
            <Self as MemoryBus>::read(self, address + 1)?,
        ];
        Ok(LittleEndian::read_u16(&bytes[..]))
    }

    /// Write little-endian u16 word from given address.
    /// May return Err if `address` or `address + 1` are not mapped to the
    /// device.
    fn write_word(&mut self, address: u16, data: u16) -> Result<(), WriteError> {
        let mut bytes = [0; 2];
        LittleEndian::write_u16(&mut bytes[..], data);
        <Self as MemoryBus>::write(self, address, bytes[0])?;
        <Self as MemoryBus>::write(self, address + 1, bytes[1])?;
        Ok(())
    }
}

#[cfg(test)]
mod test {}
