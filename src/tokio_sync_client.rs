use crate::protocol as proto;
use std::time::Duration;
use tokio_modbus::prelude::{SyncReader, SyncWriter};

type Result<T> = std::result::Result<T, crate::tokio_error::Error>;

pub struct R413D08 {
    ctx: tokio_modbus::client::sync::Context,
}

impl R413D08 {
    /// Constructs a new R413D08 client
    pub fn new(ctx: tokio_modbus::client::sync::Context) -> Self {
        Self { ctx }
    }

    /// Sets the modbus context timeout.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.ctx.set_timeout(timeout);
    }

    pub fn timeout(&self) -> Option<Duration> {
        self.ctx.timeout()
    }

    /// Read the port status of all ports.
    pub fn read_ports(&mut self) -> Result<Vec<super::State>> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::READ_PORTS_REG_ADDR, proto::READ_PORTS_REG_QUAN)??;
        Ok(rsp
            .iter()
            .map(|relay| {
                if *relay != 0 {
                    super::State::Open
                } else {
                    super::State::Close
                }
            })
            .collect::<Vec<_>>())
    }

    /// Set port to open.
    ///
    /// * 'port' - Port number 0 to 7.
    pub fn set_port_open(&mut self, port: u8) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::encode_port_number(port)?,
            proto::SET_PORT_OPEN_REG_DATA,
        )??)
    }

    /// Set all ports to open.
    pub fn set_all_open(&mut self) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::SET_ALL_PORTS_OPEN_REG_ADDR,
            proto::SET_ALL_PORTS_OPEN_REG_DATA,
        )??)
    }

    /// Set port to close.
    ///
    /// * 'port' - Port number 0 to 7.
    pub fn set_port_close(&mut self, port: u8) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::encode_port_number(port)?,
            proto::SET_PORT_CLOSE_REG_DATA,
        )??)
    }

    /// Set all ports to close.
    pub fn set_all_close(&mut self) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::SET_ALL_PORTS_CLOSE_REG_ADDR,
            proto::SET_ALL_PORTS_CLOSE_REG_DATA,
        )??)
    }

    /// Toggle port status.
    ///
    /// * 'port' - Port number 0 to 7.
    pub fn set_port_toggle(&mut self, port: u8) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::encode_port_number(port)?,
            proto::SET_PORT_TOGGLE_REG_DATA,
        )??)
    }

    /// Set port to low an all others to high.
    ///
    /// * 'port' - Port number 0 to 7.
    pub fn set_port_latch(&mut self, port: u8) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::encode_port_number(port)?,
            proto::SET_PORT_LATCH_REG_DATA,
        )??)
    }

    /// Set port to low for 1 second.
    ///
    /// * 'port' - Port number 0 to 7.
    pub fn set_port_momentary(&mut self, port: u8) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::encode_port_number(port)?,
            proto::SET_PORT_MOMENTARY_REG_DATA,
        )??)
    }

    /// Set port to low for delay seconds.
    ///
    /// * 'port' - Port number 0 to 7.
    /// * 'delay' - Delay in seconds from 0 to 255.
    pub fn set_port_delay(&mut self, port: u8, delay: u8) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::encode_port_number(port)?,
            proto::SET_PORT_DELAY_REG_DATA + delay as u16,
        )??)
    }

    /// Reads the current Modbus address
    ///
    /// Note: When using this command, only one temperature module can be connected to the RS485 bus,
    /// more than one will be wrong!
    /// The connected modbus address must be the broadcast address 255.
    pub fn read_address(&mut self) -> Result<u8> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::READ_ADDRESS_REG_ADDR, proto::READ_ADDRESS_REG_QUAN)??;
        Ok(*rsp.first().expect("Result on success expected") as u8)
    }

    /// Set the Modbus address
    ///
    /// * 'address' - The address can be from 1 to 247.
    pub fn set_address(&mut self, address: u8) -> Result<()> {
        self.ctx.write_single_register(
            proto::WRITE_ADDRESS_REG_ADDR,
            proto::write_address_encode_address(address)?,
        )??;
        Ok(())
    }
}
