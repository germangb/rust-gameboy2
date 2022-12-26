#[derive(Debug, PartialEq, Eq)]
pub enum Component {
    /// Audio Processing Unit.
    APU,

    /// Serial interface.
    Serial,
}

/// Errors are defied for the purposes of spotting/debugging emulation bugs.
/// In a real-life emulator, these should never happen!
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// IO Read error.
    #[error("TODO")]
    Read(#[from] ReadError),

    /// IO Write error.
    #[error("TODO")]
    Write(#[from] WriteError),

    /// CPU handles an unknown opcode.
    #[error("Unknown opcode ({0:02X}) (potential emulation bug)")]
    UnknownOp(u8),

    /// ROM attempted to overflow/underflow the stack pointer.
    #[error("Stack Overflow")]
    StackOverflow,

    /// ROM attempted to overflow/underflow the program counter.
    #[error("Program Counter Overflow")]
    ProgramCounterOverflow,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ReadError {
    #[error("Unknown address {0:02X} (potential emulation bug)")]
    UnknownAddr(u16),

    // Ideally I shouldn't need this error. Hopefully it can be deleted some day...
    #[error("Not yet implemented address {0:02X}")]
    AddrNotImpl(u16, Option<Component>),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WriteError {
    #[error("Unknown address {0:02X} data {1:02X} (potential emulation bug)")]
    UnknownAddr(u16, u8),

    /// Some games have writes to ROM memory. Some may be legitimate due to
    /// special chips being used in cartridge. Other times I don't know the
    /// reason (tetris is an examples).
    #[error("ROM write address {0:02X} data {1:02X}")]
    ROMAddress(u16, u8),

    #[error("Not yet implemented address {0:02X} data {1:02X}")]
    AddrNotImpl(u16, u8, Option<Component>),

    /// Invalid/Unexpected data write. Mostly here because I'm detecting odd
    /// write vales to the interrupt registers...
    #[error("Invalid write address {0:04X} data {1:02X}")]
    InvalidData(u16, u8),
}
