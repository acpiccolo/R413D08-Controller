//! A command-line interface (CLI) application for controlling an R413D08
//! 8-channel relay module via Modbus RTU (Serial) or Modbus TCP.
//!
//! This tool allows reading relay statuses, controlling individual or all relays
//! using various modes (On, Off, Toggle, Latch, Momentary, Delay), and managing
//! the device's Modbus address. It uses the `r413d08_lib` crate which provides
//! the necessary protocol definitions and a synchronous Modbus client.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use dialoguer::Confirm;
use flexi_logger::{Logger, LoggerHandle};
use log::*;
use r413d08_lib::{protocol as proto, tokio_sync_client::R413D08};
use std::{ops::Deref, panic, time::Duration};

fn default_device_name() -> String {
    if cfg!(target_os = "windows") {
        String::from("COM1")
    } else {
        String::from("/dev/ttyUSB0")
    }
}

fn parse_relay(s: &str) -> Result<proto::Port, String> {
    proto::Port::try_from(clap_num::maybe_hex::<u8>(s)?).map_err(|e| format!("{e}"))
}

fn parse_address(s: &str) -> Result<proto::Address, String> {
    proto::Address::try_from(clap_num::maybe_hex::<u8>(s)?).map_err(|e| format!("{e}"))
}

/// Defines the connection type and parameters (Modbus TCP or RTU).
#[derive(Subcommand, Debug, Clone, PartialEq)]
enum CliConnection {
    /// Connect via Modbus/TCP.
    Tcp {
        /// TCP address (e.g. 192.168.0.222:502)
        address: String,

        #[command(subcommand)]
        command: CliCommands,
    },
    /// Connect via Modbus/RTU (Serial).
    Rtu {
        /// The serial device path.
        #[arg(short, long, default_value_t = default_device_name())]
        device: String,

        /// RS485 address from 1 to 247
        #[arg(short, long, default_value_t = proto::Address::default(), value_parser = parse_address)]
        address: proto::Address,

        #[command(subcommand)]
        command: CliCommands,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum CliCommands {
    /// Read and display the current state (ON/OFF) of all 8 relays.
    Status,

    /// Turn a specific relay ON (Close circuit).
    On {
        /// Relay number 0 to 7
        #[arg(value_parser = parse_relay)]
        relay: proto::Port,
    },

    /// Turn a specific relay OFF (Open circuit).
    Off {
        /// Relay number 0 to 7
        #[arg(value_parser = parse_relay)]
        relay: proto::Port,
    },

    /// Toggle the state of a specific relay (ON->OFF, OFF->ON).
    Toggle {
        /// Relay number 0 to 7
        #[arg(value_parser = parse_relay)]
        relay: proto::Port,
    },

    /// Turn all 8 relays ON simultaneously.
    AllOn,

    /// Turn all 8 relays OFF simultaneously.
    AllOff,

    /// Latch a relay ON and turn all other relays OFF (Inter-locking).
    Latch {
        /// Relay number 0 to 7
        #[arg(value_parser = parse_relay)]
        relay: proto::Port,
    },

    /// Turn a relay ON momentarily (~1 second), then automatically OFF (Non-locking).
    Momentary {
        /// Relay number 0 to 7
        #[arg(value_parser = parse_relay)]
        relay: proto::Port,
    },

    /// Turn a relay ON, then automatically OFF after a specified delay.
    Delay {
        /// Relay number 0 to 7
        #[arg(value_parser = parse_relay)]
        relay: proto::Port,
        /// Delay duration in seconds (0-255).
        delay: u8,
    },

    /// Query the device's current Modbus address.
    /// IMPORTANT: Ensure only ONE device is connected to the bus!
    QueryAddress,

    /// Set a new Modbus address for the device.
    /// The new address must be unique on the bus. Requires addressing the device with its CURRENT address.
    SetAddress {
        /// The new Modbus address (1-247) or hex (0x01-0xF7).
        #[arg(value_parser = parse_address)]
        address: proto::Address,
    },
}

const fn about_text() -> &'static str {
    "A command-line tool to control R413D08 8-channel relay modules via Modbus TCP or RTU."
}

#[derive(Parser, Debug)]
#[command(version, about=about_text(), long_about = None)]
struct CliArgs {
    /// Verbosity level (-v, -vv, -vvv).
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

    /// Connection type (TCP or RTU) and associated command.
    #[command(subcommand)]
    pub connection: CliConnection,

    /// Modbus I/O timeout duration (e.g., "200ms", "1s").
    #[arg(value_parser = humantime::parse_duration, long, default_value = "200ms")]
    timeout: Duration,
}

fn logging_init(loglevel: LevelFilter) -> LoggerHandle {
    let log_handle = Logger::try_with_env_or_str(loglevel.as_str())
        .expect("Cannot initialize logging")
        .start()
        .expect("Cannot start logging");

    panic::set_hook(Box::new(|panic_info| {
        let (filename, line, column) = panic_info
            .location()
            .map(|loc| (loc.file(), loc.line(), loc.column()))
            .unwrap_or(("<unknown>", 0, 0));
        let cause = panic_info
            .payload()
            .downcast_ref::<String>()
            .map(String::deref)
            .or_else(|| panic_info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("<cause unknown>");

        error!(
            "Thread '{}' panicked at {}:{}:{}: {}",
            std::thread::current().name().unwrap_or("<unknown>"),
            filename,
            line,
            column,
            cause
        );
    }));
    log_handle
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    let _log_handle = logging_init(args.verbose.log_level_filter());

    let (mut client, command) = match &args.connection {
        CliConnection::Tcp { address, command } => {
            let socket_addr = address
                .parse()
                .with_context(|| format!("Cannot parse TCP address '{}'", address))?;
            trace!("Connecting via TCP to {}...", socket_addr);
            (
                R413D08::new(
                    tokio_modbus::client::sync::tcp::connect(socket_addr)
                        .with_context(|| format!("Cannot open {:?}", socket_addr))?,
                ),
                command,
            )
        }
        CliConnection::Rtu {
            device,
            address,
            command,
        } => {
            let address = if command == &CliCommands::QueryAddress {
                println!("Ensure ONLY ONE device is connected to the RS485 bus.");
                let confirmation = Confirm::new()
                    .with_prompt("Do you want to continue?")
                    .default(false)
                    .show_default(true)
                    .interact()
                    .context("Failed to get user confirmation")?;
                if !confirmation {
                    return Ok(());
                }
                let broadcast_address = proto::Address::BROADCAST;
                if address != &broadcast_address {
                    info!(
                        "Ignore address {} use broadcast address {}",
                        address, broadcast_address
                    );
                }
                broadcast_address
            } else {
                *address
            };
            trace!("Connecting via RTU to {} address {}", device, address);
            (
                R413D08::new(
                    tokio_modbus::client::sync::rtu::connect_slave(
                        &r413d08_lib::tokio_serial::serial_port_builder(device),
                        tokio_modbus::Slave(*address),
                    )
                    .with_context(|| format!("Cannot open RTU device {}", device))?,
                ),
                command,
            )
        }
    };
    client.set_timeout(Some(args.timeout));

    match command {
        CliCommands::Status => {
            let rsp = client
                .read_ports()
                .context("Failed to read port status")??;
            println!("Relay Status:");
            for (idx, state) in rsp.iter().enumerate() {
                println!(
                    "  Relay {}: {}",
                    idx,
                    if state == &proto::PortState::Close {
                        "OFF"
                    } else {
                        "ON"
                    }
                );
            }
        }
        CliCommands::On { relay } => {
            client
                .set_port_open(*relay)
                .with_context(|| format!("Failed to turn ON relay {}", **relay))??;
            println!("Relay {} turned ON", **relay);
        }
        CliCommands::AllOn => {
            client
                .set_all_open()
                .context("Failed to turn ALL relays ON")??;
            println!("All relays turned ON");
        }
        CliCommands::Off { relay } => {
            client
                .set_port_close(*relay)
                .with_context(|| format!("Failed to turn OFF relay {}", **relay))??;
            println!("Relay {} turned OFF", **relay);
        }
        CliCommands::AllOff => {
            client
                .set_all_close()
                .context("Failed to turn ALL relays OFF")??;
            println!("All relays turned OFF");
        }
        CliCommands::Toggle { relay } => {
            client
                .set_port_toggle(*relay)
                .with_context(|| format!("Failed to toggle relay {}", **relay))??;
            println!("Relay {} toggled", **relay);
        }
        CliCommands::Latch { relay } => {
            client
                .set_port_latch(*relay)
                .with_context(|| format!("Failed to latch relay {}", **relay))??;
            println!("Relay {} latched ON (others OFF)", **relay);
        }
        CliCommands::Momentary { relay } => {
            client
                .set_port_momentary(*relay)
                .with_context(|| format!("Failed to activate momentary relay {}", **relay))??;
            println!("Relay {} activated momentarily", **relay);
        }
        CliCommands::Delay { relay, delay } => {
            client
                .set_port_delay(*relay, *delay)
                .with_context(|| format!("Failed to set delay for relay {}", **relay))??;
            println!(
                "Relay {} activated with {} second delay before turning OFF",
                **relay, delay
            );
        }
        CliCommands::QueryAddress => {
            // Note: Connection was already set up with broadcast address above
            let address = client.read_address().context(
                "Failed to query device address (ensure only one device is connected)",
            )??;
            println!("Device responded with address: {}", address);
        }
        CliCommands::SetAddress { address } => {
            client
                .set_address(*address)
                .with_context(|| format!("Failed to set new Modbus address to {}", address))??;
            println!(
                "Successfully sent command to set Modbus address to {}. \
                 Remember to use this new address for future communication.",
                address
            );
        }
    }

    Ok(())
}
