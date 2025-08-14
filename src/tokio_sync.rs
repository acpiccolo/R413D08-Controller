//! Provides a synchronous Modbus client for the R413D08 relay module.
//!
//! This module defines the [`R413D08`] struct, which acts as a high-level interface
//! for interacting with the R413D08 device over Modbus (RTU or TCP). It utilizes
//! the `tokio_modbus` library in its synchronous mode and relies on protocol
//! definitions (register addresses, data encoding/decoding) from the [`crate::protocol`] module.
//!
//! # Example
//!
//! ```no_run
//! use r413d08_lib::{
//!     protocol::{Address, Port},
//!     tokio_sync::R413D08,
//! };
//! use tokio_modbus::client::sync::tcp;
//! use tokio_modbus::prelude::*;
//! use tokio_modbus::Slave;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to a Modbus TCP device
//! let socket_addr = "127.0.0.1:502".parse()?;
//! let mut ctx = tcp::connect(socket_addr)?;
//!
//! // Set the slave address of the device
//! ctx.set_slave(Slave(*Address::default()));
//!
//! // Use the stateless R413D08 functions
//! let status = R413D08::read_ports(&mut ctx)?;
//! println!("Port status: {}", status);
//! R413D08::set_port_open(&mut ctx, Port::try_from(0)?)?;
//! Ok(())
//! # }
//! ```

use crate::{protocol as proto, tokio_common::Result};
use tokio_modbus::prelude::{SyncReader, SyncWriter};

/// A synchronous client for interacting with an R413D08 relay module over Modbus.
///
/// This client wraps a [`tokio_modbus::client::sync::Context`] and provides
/// methods specific to the R413D08's protocol, such as reading port states,
/// controlling individual or all ports, and managing the device's Modbus address.
///
/// It simplifies interaction by translating device-specific operations into
/// appropriate Modbus function calls (primarily Read Holding Registers 0x03 and
/// Write Single Register 0x06) using constants and helpers defined in [`crate::protocol`].
pub struct R413D08 {}

impl R413D08 {
    /// Helper function to map tokio result to our result.
    fn map_tokio_result<T>(result: tokio_modbus::Result<T>) -> Result<T> {
        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(err)) => Err(err.into()), // Modbus exception
            Err(err) => Err(err.into()),     // IO error
        }
    }

    /// Helper function to read holding registers and decode them into a specific type.
    fn read_and_decode<T, F>(
        ctx: &mut tokio_modbus::client::sync::Context,
        address: u16,
        quantity: u16,
        decoder: F,
    ) -> Result<T>
    where
        F: FnOnce(&[u16]) -> Result<T>,
    {
        decoder(&Self::map_tokio_result(
            ctx.read_holding_registers(address, quantity),
        )?)
    }

    /// Reads the current status (Open/Close) of all [`proto::NUMBER_OF_PORTS`] ports.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// * `Ok(proto::PortStates)`: The decoded states of all ports.
    /// * `Err(tokio_modbus::Error)`: If a Modbus communication error occurs (e.g., timeout, CRC error, exception response).
    pub fn read_ports(ctx: &mut tokio_modbus::client::sync::Context) -> Result<proto::PortStates> {
        Self::read_and_decode(
            ctx,
            proto::PortStates::ADDRESS,
            proto::PortStates::QUANTITY,
            |words| Ok(proto::PortStates::decode_from_holding_registers(words)),
        )
    }

    /// Sets the specified port to the **Open** state (activates relay).
    ///
    /// # Arguments
    ///
    /// * `port`: The [`proto::Port`] to open.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub fn set_port_open(
        ctx: &mut tokio_modbus::client::sync::Context,
        port: proto::Port,
    ) -> Result<()> {
        Self::map_tokio_result(ctx.write_single_register(
            port.address_for_write_register(),
            proto::Port::REG_DATA_SET_PORT_OPEN,
        ))
    }

    /// Sets **all** ports to the **Open** state simultaneously.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub fn set_all_open(ctx: &mut tokio_modbus::client::sync::Context) -> Result<()> {
        Self::map_tokio_result(ctx.write_single_register(
            proto::PortsAll::ADDRESS,
            proto::PortsAll::REG_DATA_SET_ALL_OPEN,
        ))
    }

    /// Sets the specified port to the **Close** state (deactivates relay).
    ///
    /// # Arguments
    ///
    /// * `port`: The [`proto::Port`] to close.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub fn set_port_close(
        ctx: &mut tokio_modbus::client::sync::Context,
        port: proto::Port,
    ) -> Result<()> {
        Self::map_tokio_result(ctx.write_single_register(
            port.address_for_write_register(),
            proto::Port::REG_DATA_SET_PORT_CLOSE,
        ))
    }

    /// Sets **all** ports to the **Close** state simultaneously.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub fn set_all_close(ctx: &mut tokio_modbus::client::sync::Context) -> Result<()> {
        Self::map_tokio_result(ctx.write_single_register(
            proto::PortsAll::ADDRESS,
            proto::PortsAll::REG_DATA_SET_ALL_CLOSE,
        ))
    }

    /// Toggles the current state of the specified port (Open -> Close, Close -> Open). Also called "Self-locking".
    ///
    /// # Arguments
    ///
    /// * `port`: The [`proto::Port`] to toggle.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub fn set_port_toggle(
        ctx: &mut tokio_modbus::client::sync::Context,
        port: proto::Port,
    ) -> Result<()> {
        Self::map_tokio_result(ctx.write_single_register(
            port.address_for_write_register(),
            proto::Port::REG_DATA_SET_PORT_TOGGLE,
        ))
    }

    /// Latches the specified port (Inter-locking): Sets the given `port` to Open and all *other* ports to Close.
    ///
    /// # Arguments
    ///
    /// * `port`: The [`proto::Port`] to latch open.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub fn set_port_latch(
        ctx: &mut tokio_modbus::client::sync::Context,
        port: proto::Port,
    ) -> Result<()> {
        Self::map_tokio_result(ctx.write_single_register(
            port.address_for_write_register(),
            proto::Port::REG_DATA_SET_PORT_LATCH,
        ))
    }

    /// Activates the specified port momentarily (Non-locking): Opens the port for ~1 second, then automatically Closes.
    ///
    /// # Arguments
    ///
    /// * `port`: The [`proto::Port`] to activate momentarily.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub fn set_port_momentary(
        ctx: &mut tokio_modbus::client::sync::Context,
        port: proto::Port,
    ) -> Result<()> {
        Self::map_tokio_result(ctx.write_single_register(
            port.address_for_write_register(),
            proto::Port::REG_DATA_SET_PORT_MOMENTARY,
        ))
    }

    /// Initiates a delayed action on the specified port (typically Open -> Delay -> Close).
    ///
    /// # Arguments
    ///
    /// * `port`: The [`proto::Port`] for the delayed action.
    /// * `delay`: Delay duration in seconds (0-255).
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub fn set_port_delay(
        ctx: &mut tokio_modbus::client::sync::Context,
        port: proto::Port,
        delay: u8,
    ) -> Result<()> {
        Self::map_tokio_result(ctx.write_single_register(
            port.address_for_write_register(),
            proto::Port::encode_delay_for_write_register(delay),
        ))
    }

    /// Reads the configured Modbus device address from the device itself.
    ///
    /// **Important Usage Notes:**
    /// * This command typically requires the client's context to be set
    ///   to the **correct current address** of the target device OR the
    ///   **broadcast address** ([`proto::Address::BROADCAST`]).
    /// * Using the broadcast address usually requires that **only one** device is
    ///   physically present and responding on the Modbus network segment.
    /// * Consult the R413D08 device manual for specifics on reading the address.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// * `Ok(proto::Address)`: The decoded device address.
    /// * `Err(tokio_modbus::Error)`: If a Modbus communication error occurs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use r413d08_lib::tokio_sync::R413D08;
    /// use r413d08_lib::protocol::Address;
    /// use tokio_modbus::client::sync::Context;
    /// use tokio_modbus::Slave;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Requires serial port features enabled in tokio-modbus
    /// let builder = tokio_serial::new("/dev/ttyUSB0", 9600) // Baud rate 9600
    ///     .parity(tokio_serial::Parity::None)
    ///     .stop_bits(tokio_serial::StopBits::One)
    ///     .data_bits(tokio_serial::DataBits::Eight)
    ///     .flow_control(tokio_serial::FlowControl::None);
    /// // Assume only one device connected, use broadcast address for reading
    /// let slave = Slave(*Address::BROADCAST);
    /// let mut ctx = tokio_modbus::client::sync::rtu::connect_slave(&builder, slave).expect("Failed to connect");
    ///
    /// let address = R413D08::read_address(&mut ctx)?;
    /// println!("Device responded with address: {}", address);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_address(ctx: &mut tokio_modbus::client::sync::Context) -> Result<proto::Address> {
        Self::read_and_decode(
            ctx,
            proto::Address::ADDRESS,
            proto::Address::QUANTITY,
            |words| Ok(proto::Address::decode_from_holding_registers(words)?),
        )
    }

    /// Sets a new Modbus device address.
    ///
    /// **Warning:**
    /// * This permanently changes the device's Modbus address.
    /// * This command must be sent while addressing the device using its **current** Modbus address.
    /// * After successfully changing the address, subsequent communication with the
    ///   device **must** use the new address.
    ///
    /// # Arguments
    ///
    /// * `address`: The new [`proto::Address`] to assign to the device.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use r413d08_lib::tokio_sync::R413D08;
    /// use r413d08_lib::protocol::Address;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let socket_addr = "192.168.1.100:502".parse()?;
    /// # let mut ctx = tokio_modbus::client::sync::tcp::connect(socket_addr)?;
    ///
    /// // Set the new Modbus address to 10.
    /// let new_address = Address::try_from(10)?;
    /// R413D08::set_address(&mut ctx, new_address)?;
    /// println!("Address successfully changed to {}", new_address);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_address(
        ctx: &mut tokio_modbus::client::sync::Context,
        address: proto::Address,
    ) -> Result<()> {
        Self::map_tokio_result(
            ctx.write_single_register(proto::Address::ADDRESS, address.encode_for_write_register()),
        )
    }
}
