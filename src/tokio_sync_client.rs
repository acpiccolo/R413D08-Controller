//! Provides a synchronous Modbus client for the R413D08 relay module.
//!
//! This module defines the [`R413D08`] struct, which acts as a high-level interface
//! for interacting with the R413D08 device over Modbus (RTU or TCP). It utilizes
//! the `tokio_modbus` library in its synchronous mode and relies on protocol
//! definitions (register addresses, data encoding/decoding) from the [`crate::protocol`] module.

use crate::{protocol as proto, tokio_common::Result};
use std::time::Duration;
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
pub struct R413D08 {
    /// The underlying synchronous Modbus client context.
    ctx: tokio_modbus::client::sync::Context,
}

impl R413D08 {
    /// Creates a new R413D08 client instance.
    ///
    /// Requires a pre-configured [`tokio_modbus::client::sync::Context`], which
    /// encapsulates the Modbus connection (TCP or Serial).
    ///
    /// # Arguments
    ///
    /// * `ctx`: A synchronous Modbus client context, already connected.
    ///
    /// # Examples
    ///
    /// **TCP Connection:**
    /// ```no_run
    /// use r413d08_lib::tokio_sync_client::R413D08;
    /// use r413d08_lib::protocol::{Address, Port};
    /// use tokio_modbus::client::sync::Context;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Connect to a Modbus TCP device at IP 192.168.1.100, port 502
    /// let socket_addr = "192.168.1.100:502".parse()?;
    /// let mut ctx = tokio_modbus::client::sync::tcp::connect(socket_addr)?;
    ///
    /// let mut client = R413D08::new(ctx);
    ///
    /// // Now use the client methods
    /// let port_states = client.read_ports()?;
    /// println!("Port states: {}", port_states);
    /// client.set_port_open(Port::try_from(0)?)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// **Serial (RTU) Connection (Conceptual):**
    /// ```no_run
    /// use r413d08_lib::tokio_sync_client::R413D08;
    /// use r413d08_lib::protocol::{Address, Port};
    /// use tokio_modbus::client::sync::Context;
    /// use tokio_modbus::Slave;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let builder = r413d08_lib::tokio_common::serial_port_builder(
    ///     "/dev/ttyUSB0", // Or "COM3" on Windows, etc.
    /// );
    /// // Assume device is currently at address 1
    /// let slave = Slave(*Address::try_from(1)?);
    /// let mut ctx = tokio_modbus::client::sync::rtu::connect_slave(&builder, slave).expect("Failed to connect");
    /// let mut client = R413D08::new(ctx);
    /// // ... use client ...
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(ctx: tokio_modbus::client::sync::Context) -> Self {
        Self { ctx }
    }

    /// Sets the timeout for subsequent Modbus read/write operations.
    ///
    /// If an operation takes longer than this duration, it will fail with a timeout error
    /// (specifically, a `ModbusError` with `ErrorKind::Timeout`).
    ///
    /// # Arguments
    ///
    /// * `timeout` - The `Duration` to wait before timing out. Can also accept `Option<Duration>`.
    ///   Passing `None` disables the timeout.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r413d08_lib::tokio_sync_client::R413D08;
    /// # use std::time::Duration;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let socket_addr = "192.168.1.100:502".parse()?;
    /// # let mut ctx = tokio_modbus::client::sync::tcp::connect(socket_addr)?;
    /// let mut client = R413D08::new(ctx);
    /// // Set a 2-second timeout
    /// client.set_timeout(Some(Duration::from_secs(2)));
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_timeout(&mut self, timeout: impl Into<Option<Duration>>) {
        self.ctx.set_timeout(timeout);
    }

    /// Retrieves the current Modbus communication timeout.
    ///
    /// # Returns
    ///
    /// An `Option<Duration>` indicating the configured timeout duration,
    /// or `None` if no timeout is set.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r413d08_lib::tokio_sync_client::R413D08;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let socket_addr = "192.168.1.100:502".parse()?;
    /// # let mut ctx = tokio_modbus::client::sync::tcp::connect(socket_addr)?;
    /// let client = R413D08::new(ctx);
    /// if let Some(timeout) = client.timeout() {
    ///     println!("Current timeout: {:?}", timeout);
    /// } else {
    ///     println!("No timeout is set.");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn timeout(&self) -> Option<Duration> {
        self.ctx.timeout()
    }

    /// Helper function to map tokio result to our result.
    fn map_tokio_result<T>(result: tokio_modbus::Result<T>) -> Result<T> {
        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(err)) => Err(err.into()), // Modbus exception
            Err(err) => Err(err.into()),     // IO error
        }
    }

    /// Helper function to read holding registers and decode them into a specific type.
    fn read_and_decode<T, F>(&mut self, address: u16, quantity: u16, decoder: F) -> Result<T>
    where
        F: FnOnce(&[u16]) -> Result<T>,
    {
        decoder(&Self::map_tokio_result(
            self.ctx.read_holding_registers(address, quantity),
        )?)
    }

    /// Reads the current status (Open/Close) of all [`proto::NUMBER_OF_PORTS`] ports.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// * `Ok(proto::PortStates)`: The decoded states of all ports.
    /// * `Err(tokio_modbus::Error)`: If a Modbus communication error occurs (e.g., timeout, CRC error, exception response).
    pub fn read_ports(&mut self) -> Result<proto::PortStates> {
        self.read_and_decode(
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
    pub fn set_port_open(&mut self, port: proto::Port) -> Result<()> {
        Self::map_tokio_result(self.ctx.write_single_register(
            port.address_for_write_register(),
            proto::Port::REG_DATA_SET_PORT_OPEN,
        ))
    }

    /// Sets **all** ports to the **Open** state simultaneously.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub fn set_all_open(&mut self) -> Result<()> {
        Self::map_tokio_result(self.ctx.write_single_register(
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
    pub fn set_port_close(&mut self, port: proto::Port) -> Result<()> {
        Self::map_tokio_result(self.ctx.write_single_register(
            port.address_for_write_register(),
            proto::Port::REG_DATA_SET_PORT_CLOSE,
        ))
    }

    /// Sets **all** ports to the **Close** state simultaneously.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub fn set_all_close(&mut self) -> Result<()> {
        Self::map_tokio_result(self.ctx.write_single_register(
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
    pub fn set_port_toggle(&mut self, port: proto::Port) -> Result<()> {
        Self::map_tokio_result(self.ctx.write_single_register(
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
    pub fn set_port_latch(&mut self, port: proto::Port) -> Result<()> {
        Self::map_tokio_result(self.ctx.write_single_register(
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
    pub fn set_port_momentary(&mut self, port: proto::Port) -> Result<()> {
        Self::map_tokio_result(self.ctx.write_single_register(
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
    pub fn set_port_delay(&mut self, port: proto::Port, delay: u8) -> Result<()> {
        Self::map_tokio_result(self.ctx.write_single_register(
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
    /// use r413d08_lib::tokio_sync_client::R413D08;
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
    /// let mut client = R413D08::new(ctx);
    ///
    /// let address = client.read_address()?;
    /// println!("Device responded with address: {}", address);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_address(&mut self) -> Result<proto::Address> {
        self.read_and_decode(proto::Address::ADDRESS, proto::Address::QUANTITY, |words| {
            Ok(proto::Address::decode_from_holding_registers(words)?)
        })
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
    /// use r413d08_lib::tokio_sync_client::R413D08;
    /// use r413d08_lib::protocol::Address;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let socket_addr = "192.168.1.100:502".parse()?;
    /// # let mut ctx = tokio_modbus::client::sync::tcp::connect(socket_addr)?;
    /// let mut client = R413D08::new(ctx);
    ///
    /// // Set the new Modbus address to 10.
    /// let new_address = Address::try_from(10)?;
    /// client.set_address(new_address)?;
    /// println!("Address successfully changed to {}", new_address);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_address(&mut self, address: proto::Address) -> Result<()> {
        Self::map_tokio_result(
            self.ctx.write_single_register(
                proto::Address::ADDRESS,
                address.encode_for_write_register(),
            ),
        )
    }
}
