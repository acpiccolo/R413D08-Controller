//! Provides an asynchronous Modbus client for the R413D08 relay module.
//!
//! This module defines the [`R413D08`] struct, which acts as a high-level interface
//! for interacting with the R413D08 device over Modbus (RTU or TCP). It utilizes
//! the `tokio_modbus` library in its asynchronous mode and relies on protocol
//! definitions (register addresses, data encoding/decoding) from the [`crate::protocol`] module.

use crate::protocol as proto;
use tokio_modbus::prelude::{Reader, Writer};

/// An asynchronous client for interacting with an R413D08 relay module over Modbus.
///
/// This client wraps a [`tokio_modbus::client::Context`] and provides
/// methods specific to the R413D08's protocol, such as reading port states,
/// controlling individual or all ports, and managing the device's Modbus address.
///
/// It simplifies interaction by translating device-specific operations into
/// appropriate Modbus function calls (primarily Read Holding Registers 0x03 and
/// Write Single Register 0x06) using constants and helpers defined in [`crate::protocol`].
pub struct R413D08 {
    /// The underlying asynchronous Modbus client context.
    ctx: tokio_modbus::client::Context,
}

impl R413D08 {
    /// Creates a new R413D08 client instance.
    ///
    /// Requires a pre-configured [`tokio_modbus::client::Context`], which
    /// encapsulates the Modbus connection (TCP or Serial).
    ///
    /// # Arguments
    ///
    /// * `ctx`: An asynchronous Modbus client context, already connected.
    ///
    /// # Examples
    ///
    /// **TCP Connection:**
    /// ```no_run
    /// use r413d08_lib::tokio_async_client::R413D08;
    /// use r413d08_lib::protocol::{Address, Port};
    /// use tokio_modbus::client::Context;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Connect to a Modbus TCP device at IP 192.168.1.100, port 502
    /// let socket_addr = "192.168.1.100:502".parse()?;
    /// let mut ctx = tokio_modbus::client::tcp::connect(socket_addr).await?;
    ///
    /// let mut client = R413D08::new(ctx);
    ///
    /// // Now use the client methods
    /// let port_states = client.read_ports().await??;
    /// println!("Port states: {}", port_states);
    /// client.set_port_open(Port::try_from(0)?).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// **Serial (RTU) Connection (Conceptual):**
    /// ```no_run
    /// use r413d08_lib::tokio_async_client::R413D08;
    /// use r413d08_lib::protocol::{Address, Port};
    /// use tokio_modbus::client::Context;
    /// use tokio_modbus::Slave;
    /// use tokio_serial::SerialStream;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Requires serial port features enabled in tokio-modbus
    /// let builder = tokio_serial::new("/dev/ttyUSB0", 9600) // Baud rate 9600
    ///    .parity(tokio_serial::Parity::None)
    ///    .stop_bits(tokio_serial::StopBits::One)
    ///    .data_bits(tokio_serial::DataBits::Eight)
    ///    .flow_control(tokio_serial::FlowControl::None);
    /// let port = SerialStream::open(&builder)?;
    /// // Assume device is currently at address 1
    /// let slave = Slave(*Address::try_from(1)?);
    /// let mut ctx = tokio_modbus::client::rtu::attach_slave(port, slave);
    /// let mut client = R413D08::new(ctx);
    /// // ... use client ...
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(ctx: tokio_modbus::client::Context) -> Self {
        Self { ctx }
    }

    /// Reads the current status (Open/Close) of all [`proto::NUMBER_OF_PORTS`] ports.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// * `Ok(proto::PortStates)`: The decoded states of all ports.
    /// * `Err(tokio_modbus::Error)`: If a Modbus communication error occurs (e.g., timeout, CRC error, exception response).
    pub async fn read_ports(&mut self) -> tokio_modbus::Result<proto::PortStates> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::PortStates::ADDRESS, proto::PortStates::QUANTITY)
            .await;
        match rsp {
            Ok(Ok(rsp)) => {
                // Modbus read successful, now decode
                Ok(Ok(proto::PortStates::decode_from_holding_registers(&rsp)))
            }
            Ok(Err(err)) => {
                // Modbus read returned an error/exception within the Ok variant
                Ok(Err(err))
            }
            Err(err) => {
                // Underlying communication error (e.g., IO error, timeout)
                Err(err)
            }
        }
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
    pub async fn set_port_open(&mut self, port: proto::Port) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                port.address_for_write_register(),
                proto::Port::REG_DATA_SET_PORT_OPEN,
            )
            .await
    }

    /// Sets **all** ports to the **Open** state simultaneously.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub async fn set_all_open(&mut self) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                proto::PortsAll::ADDRESS,
                proto::PortsAll::REG_DATA_SET_ALL_OPEN,
            )
            .await
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
    pub async fn set_port_close(&mut self, port: proto::Port) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                port.address_for_write_register(),
                proto::Port::REG_DATA_SET_PORT_CLOSE,
            )
            .await
    }

    /// Sets **all** ports to the **Close** state simultaneously.
    ///
    /// # Errors
    ///
    /// Returns `Err(tokio_modbus::Error)` if a Modbus communication error occurs.
    pub async fn set_all_close(&mut self) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                proto::PortsAll::ADDRESS,
                proto::PortsAll::REG_DATA_SET_ALL_CLOSE,
            )
            .await
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
    pub async fn set_port_toggle(&mut self, port: proto::Port) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                port.address_for_write_register(),
                proto::Port::REG_DATA_SET_PORT_TOGGLE,
            )
            .await
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
    pub async fn set_port_latch(&mut self, port: proto::Port) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                port.address_for_write_register(),
                proto::Port::REG_DATA_SET_PORT_LATCH,
            )
            .await
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
    pub async fn set_port_momentary(&mut self, port: proto::Port) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                port.address_for_write_register(),
                proto::Port::REG_DATA_SET_PORT_MOMENTARY,
            )
            .await
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
    pub async fn set_port_delay(
        &mut self,
        port: proto::Port,
        delay: u8,
    ) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                port.address_for_write_register(),
                proto::Port::encode_delay_for_write_register(delay),
            )
            .await
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
    /// * `Ok(proto::Address)`: The decoded device address. Note that the underlying [`proto::Address::decode_from_holding_registers`] does not currently validate the range of the returned address.
    /// * `Err(tokio_modbus::Error)`: If a Modbus communication error occurs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use r413d08_lib::tokio_async_client::R413D08;
    /// use r413d08_lib::protocol::{Address, Port};
    /// use tokio_modbus::client::Context;
    /// use tokio_modbus::Slave;
    /// use tokio_serial::SerialStream;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Requires serial port features enabled in tokio-modbus
    /// let builder = tokio_serial::new("/dev/ttyUSB0", 9600) // Baud rate 9600
    ///    .parity(tokio_serial::Parity::None)
    ///    .stop_bits(tokio_serial::StopBits::One)
    ///    .data_bits(tokio_serial::DataBits::Eight)
    ///    .flow_control(tokio_serial::FlowControl::None);
    /// let port = SerialStream::open(&builder)?;
    /// // Assume only one device connected, use broadcast address for reading
    /// let slave = Slave(*Address::BROADCAST);
    /// let mut ctx = tokio_modbus::client::rtu::attach_slave(port, slave);
    ///
    /// let mut client = R413D08::new(ctx);
    ///
    /// let address = client.read_address().await??;
    /// println!("Device responded with address: {}", address);
    /// # Ok(())
    /// # }
    pub async fn read_address(&mut self) -> tokio_modbus::Result<proto::Address> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::Address::ADDRESS, proto::Address::QUANTITY)
            .await;
        match rsp {
            Ok(Ok(rsp)) => {
                // Modbus read successful, decode (assuming decode doesn't fail here)
                Ok(Ok(proto::Address::decode_from_holding_registers(&rsp)))
            }
            Ok(Err(err)) => {
                // Modbus exception occurred
                Ok(Err(err))
            }
            Err(err) => {
                // IO error occurred
                Err(err)
            }
        }
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
    /// client.set_address(new_address)??;
    /// println!("Address successfully changed to {}", new_address);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_address(&mut self, address: proto::Address) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(proto::Address::ADDRESS, address.encode_for_write_register())
            .await
    }
}
