#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// Read or Write from an invalid address.
    ///
    /// # Notes
    /// Normally this shouldn't happen. If this type of error is raised, there's
    /// a bug in the emulation.
    #[error("Use of invalid address: {0:02x}h")]
    InvalidAddr(u16),

    /// Breakpoint trigger.
    #[error("Reached breakpoint.")]
    Breakpoint,
}
