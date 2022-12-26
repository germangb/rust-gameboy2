#[derive(Debug, PartialEq, Eq)]
pub enum Component {
    /// Audio Processing Unit.
    APU,

    /// Serial interface.
    Serial,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("TODO")]
    Read(#[from] ReadError),

    #[error("TODO")]
    Write(#[from] WriteError),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ReadError {
    #[error("Invalid address 0x{0:02x}")]
    InvalidAddress(u16),

    // Ideally I shouldn't need this error. Hopefully it can be deleted some day...
    #[error("Not yet implemented address 0x{0:02x}")]
    AddrNotImpl(u16, Option<Component>),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WriteError {
    #[error("Invalid address 0x{0:02x} data 0x{1:02x} (potential bug)")]
    InvalidAddress(u16, u8),

    /// Some games have writes to ROM memory. Some may be legitimate due to
    /// special chips being used in cartridge. Other times I don't know the
    /// reason (tetris is an example).
    #[error("ROM write address 0x{0:02x} data 0x{1:02x}")]
    ROMAddress(u16, u8),

    #[error("Not yet implemented address 0x{0:02x} data 0x{1:02x}")]
    AddrNotImpl(u16, u8, Option<Component>),
}
