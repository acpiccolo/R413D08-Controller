use crate::protocol;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "The port number {0} is outside the permissible range of {min} to {max}",
        min = protocol::PORT_NUMBER_MIN,
        max = protocol::PORT_NUMBER_MAX
    )]
    PortOutOfRange(u8),
    #[error(
        "The address value {0} is outside the permissible range of {min} to {max}",
        min = protocol::ADDRESS_MIN,
        max = protocol::ADDRESS_MAX
    )]
    AddressOutOfRange(u8),
}
