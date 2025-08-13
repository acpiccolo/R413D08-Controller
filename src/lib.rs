//! A library for controlling the R413D08 8-channel relay module via Modbus.
//!
//! This crate provides a high-level API for interacting with the R413D08 relay
//! module, abstracting away the low-level Modbus protocol details. It offers
//! both synchronous and asynchronous clients, built on top of the `tokio-modbus`
//! and `tokio-serial` libraries.
//!
//! ## Features
//!
//! - **Protocol Implementation**: Complete implementation of the R413D08 Modbus protocol,
//!   including commands for reading and writing relay states, and configuring the device.
//! - **Synchronous Client**: A blocking client (`r413d08_lib::tokio_sync_client::R413D08`)
//!   for straightforward, synchronous applications.
//! - **Asynchronous Client**: A non-blocking, `async/await` client (`r413d08_lib::tokio_async_client::R413D08`)
//!   for integration into `tokio`-based applications.
//! - **Strongly-Typed API**: Utilizes Rust's type system to ensure protocol correctness
//!   (e.g., `Port`, `Address`, `PortState`).
//!
//! ## Quick Start
//!
//! Here is a quick example of how to use the synchronous client to turn a relay on:
//!
//! ```no_run
//! use r413d08_lib::{
//!     protocol::{Address, Port},
//!     tokio_sync_client::R413D08,
//! };
//! use tokio_modbus::Slave;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure the serial port for RTU communication
//!     let builder = r413d08_lib::tokio_common::serial_port_builder("/dev/ttyUSB0");
//!     // Set the Modbus slave address of the device
//!     let slave = Slave(*Address::default());
//!
//!     // Connect to the device
//!     let mut ctx = tokio_modbus::client::sync::rtu::connect_slave(&builder, slave)?;
//!     let mut client = R413D08::new(ctx);
//!
//!     // Turn relay 0 ON
//!     let port_to_control = Port::try_from(0)?;
//!     client.set_port_open(port_to_control)?;
//!
//!     println!("Successfully turned on relay 0.");
//!
//!     Ok(())
//! }
//! ```
//!
//! For more details on the protocol and available commands, see the [`protocol`] module.
//! For client-specific usage, see the [`tokio_sync_client`] and [`tokio_async_client`] modules.

pub mod protocol;

#[cfg(any(
    feature = "tokio-rtu-sync",
    feature = "tokio-tcp-sync",
    feature = "tokio-rtu",
    feature = "tokio-tcp"
))]
pub mod tokio_common;

#[cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync"))]
pub mod tokio_sync_client;

#[cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))]
pub mod tokio_async_client;
