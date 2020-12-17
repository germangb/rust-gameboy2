use crate::device::Address;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Use of invalid address: {0:02x}h")]
    InvalidAddr(Address),
}
