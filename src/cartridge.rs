use crate::dev::Device;

/// Cartridge header decoder.
#[cfg(nop)]
pub struct HeaderDecoder<'a> {
    inner: &'a dyn Cartridge,
}

pub trait Cartridge: Device {
    #[cfg(nop)]
    /// Return cartridge header decoder.
    fn header(&self) -> HeaderDecoder<'_> {
        HeaderDecoder { inner: self }
    }
}
