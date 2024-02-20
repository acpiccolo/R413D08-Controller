use crate::Error;

pub const NUMBER_OF_PORTS: u8 = 8;
pub const FACTORY_DEFAULT_ADDRESS: u8 = 0x01;
pub const READ_ADDRESS_BROADCAST_ADDRESS: u8 = 0xFF;

pub const READ_PORTS_REG_ADDR: u16 = 0x0001;
pub const READ_PORTS_REG_QUAN: u16 = NUMBER_OF_PORTS as u16;

pub const SET_PORT_OPEN_REG_DATA: u16 = 0x0100;
pub const SET_PORT_CLOSE_REG_DATA: u16 = 0x0200;
pub const SET_PORT_TOGGLE_REG_DATA: u16 = 0x0300;
pub const SET_PORT_LATCH_REG_DATA: u16 = 0x0400;
pub const SET_PORT_MOMENTARY_REG_DATA: u16 = 0x0500;
pub const SET_PORT_DELAY_REG_DATA: u16 = 0x0600;

pub const SET_ALL_PORTS_OPEN_REG_ADDR: u16 = 0x0000;
pub const SET_ALL_PORTS_OPEN_REG_DATA: u16 = 0x0700;

pub const SET_ALL_PORTS_CLOSE_REG_ADDR: u16 = 0x0000;
pub const SET_ALL_PORTS_CLOSE_REG_DATA: u16 = 0x0800;

pub const READ_ADDRESS_REG_ADDR: u16 = 0x00FF;
pub const READ_ADDRESS_REG_QUAN: u16 = 0x001;
pub const WRITE_ADDRESS_REG_ADDR: u16 = 0x00FF;

pub fn encode_port_number(port: u8) -> std::result::Result<u16, Error> {
    if (0..=NUMBER_OF_PORTS - 1).contains(&port) {
        Ok((port as u16) + 1)
    } else {
        Err(Error::RangeError)
    }
}

pub fn write_address_encode_address(address: u8) -> std::result::Result<u16, Error> {
    if (1..=247).contains(&address) {
        Ok(address as u16)
    } else {
        Err(Error::RangeError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_address_encode_address_test() {
        assert!(matches!(
            write_address_encode_address(0),
            Err(Error::RangeError)
        ));
        assert!(matches!(write_address_encode_address(1), Ok(1)));
        assert!(matches!(write_address_encode_address(247), Ok(247)));
        assert!(matches!(
            write_address_encode_address(248),
            Err(Error::RangeError)
        ));
    }
}
