use crate::protocol;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "The port number {0} is outside the permissible range of {} to {}",
        protocol::PORT_NUMBER_MIN,
        protocol::PORT_NUMBER_MAX
    )]
    PortOutOfRange(u8),
    #[error(
        "The address value {0} is outside the permissible range of {} to {}",
        protocol::ADDRESS_MIN,
        protocol::ADDRESS_MAX
    )]
    AddressOutOfRange(u8),
}
