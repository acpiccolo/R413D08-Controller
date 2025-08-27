//! Provides a thread-safe, synchronous Modbus client for the R413D08 relay module.
//!
//! This module defines the [`SafeClient`] struct, which acts as a high-level,
//! stateful, and thread-safe interface for interacting with the R413D08 device.
//! It wraps a `tokio-modbus` `sync::Context` within a `std::sync::Mutex` to ensure
//! that all operations are serialized, making it safe to share across multiple
//! threads (e.g., using an `Arc<SafeClient>`).

use crate::{protocol as proto, tokio_common::Result, tokio_sync::R413D08};
use std::sync::{Arc, Mutex};
use tokio_modbus::{client::sync::Context, prelude::SlaveContext, Slave};

/// A thread-safe, synchronous client for an R413D08 relay module.
///
/// This client encapsulates a [`tokio_modbus::client::sync::Context`] within an
/// `Arc<Mutex<...>>`, allowing it to be safely shared and cloned across multiple
/// threads. All device operations are internally serialized,
/// preventing concurrent access issues.
///
/// It also provides a safer `set_address` method that automatically updates
/// the client's internal slave ID after successfully changing the device's
/// Modbus address, preventing desynchronization errors.
///
/// # Example
///
/// ```no_run
/// use r413d08_lib::{
///     protocol::Port,
///     tokio_sync_safe_client::SafeClient,
/// };
/// use std::thread;
/// use tokio_modbus::client::sync::tcp;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let socket_addr = "127.0.0.1:502".parse()?;
///     let ctx = tcp::connect(socket_addr)?;
///     let client = SafeClient::new(ctx);
///
///     // Clone the client to share it between threads
///     let client_clone = client.clone();
///     thread::spawn(move || {
///         // Use the client in another thread
///         client_clone.set_port_open(Port::try_from(1).unwrap()).unwrap();
///     });
///
///     // Use the client in the main thread
///     let status = client.read_ports()?;
///     println!("Port status: {}", status);
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct SafeClient {
    ctx: Arc<Mutex<Context>>,
}

impl SafeClient {
    /// Creates a new `SafeClient` instance.
    ///
    /// # Arguments
    ///
    /// * `ctx`: A synchronous Modbus client context, already connected.
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx: Arc::new(Mutex::new(ctx)),
        }
    }

    /// Creates a new `SafeClient` from an existing `Arc<Mutex<Context>>`.
    ///
    /// This allows multiple `SafeClient` instances to share the exact same
    /// underlying connection context.
    pub fn from_shared(ctx: Arc<Mutex<Context>>) -> Self {
        Self { ctx }
    }

    /// Clones and returns the underlying `Arc<Mutex<Context>>`.
    ///
    /// This allows the shared context to be used by other parts of an
    /// application that may need direct access to the Modbus context.
    pub fn clone_shared(&self) -> Arc<Mutex<Context>> {
        self.ctx.clone()
    }

    /// Reads the current status (Open/Close) of all ports.
    pub fn read_ports(&self) -> Result<proto::PortStates> {
        let mut guard = self.ctx.lock().unwrap();
        R413D08::read_ports(&mut guard)
    }

    /// Sets the specified port to the **Open** state.
    pub fn set_port_open(&self, port: proto::Port) -> Result<()> {
        let mut guard = self.ctx.lock().unwrap();
        R413D08::set_port_open(&mut guard, port)
    }

    /// Sets **all** ports to the **Open** state.
    pub fn set_all_open(&self) -> Result<()> {
        let mut guard = self.ctx.lock().unwrap();
        R413D08::set_all_open(&mut guard)
    }

    /// Sets the specified port to the **Close** state.
    pub fn set_port_close(&self, port: proto::Port) -> Result<()> {
        let mut guard = self.ctx.lock().unwrap();
        R413D08::set_port_close(&mut guard, port)
    }

    /// Sets **all** ports to the **Close** state.
    pub fn set_all_close(&self) -> Result<()> {
        let mut guard = self.ctx.lock().unwrap();
        R413D08::set_all_close(&mut guard)
    }

    /// Toggles the current state of the specified port.
    pub fn set_port_toggle(&self, port: proto::Port) -> Result<()> {
        let mut guard = self.ctx.lock().unwrap();
        R413D08::set_port_toggle(&mut guard, port)
    }

    /// Latches the specified port (opens it and closes all others).
    pub fn set_port_latch(&self, port: proto::Port) -> Result<()> {
        let mut guard = self.ctx.lock().unwrap();
        R413D08::set_port_latch(&mut guard, port)
    }

    /// Activates the specified port momentarily.
    pub fn set_port_momentary(&self, port: proto::Port) -> Result<()> {
        let mut guard = self.ctx.lock().unwrap();
        R413D08::set_port_momentary(&mut guard, port)
    }

    /// Activates the specified port with a delayed close.
    pub fn set_port_delay(&self, port: proto::Port, delay: u8) -> Result<()> {
        let mut guard = self.ctx.lock().unwrap();
        R413D08::set_port_delay(&mut guard, port, delay)
    }

    /// Reads the configured Modbus device address.
    ///
    /// It's recommended to use the broadcast address for this operation,
    /// ensuring only one device is on the bus.
    pub fn read_address(&self) -> Result<proto::Address> {
        let mut guard = self.ctx.lock().unwrap();
        R413D08::read_address(&mut guard)
    }

    /// Sets a new Modbus device address.
    ///
    /// **This method is safer than the stateless equivalent.** Upon successfully
    /// changing the device's address, it automatically updates the client's
    /// internal slave ID to match. This keeps the client synchronized with the
    /// device state, preventing subsequent communication errors.
    pub fn set_address(&self, address: proto::Address) -> Result<()> {
        let mut guard = self.ctx.lock().unwrap();
        R413D08::set_address(&mut guard, address)?;
        guard.set_slave(Slave(*address));
        Ok(())
    }
}
