use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clap_num::maybe_hex;
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

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum CliConnection {
    /// Use Modbus/TCP connection
    Tcp {
        // TCP address (e.g. 192.168.0.222:502)
        address: String,

        #[command(subcommand)]
        command: CliCommands,
    },
    /// Use Modbus/RTU connection
    Rtu {
        /// Device
        #[arg(short, long, default_value_t = default_device_name())]
        device: String,

        /// RS485 address from 1 to 247
        #[arg(short, long, default_value_t = proto::FACTORY_DEFAULT_ADDRESS, value_parser=maybe_hex::<u8>)]
        address: u8,

        #[command(subcommand)]
        command: CliCommands,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum CliCommands {
    /// Read the current state of all relays
    Status,

    /// Set a relay to on
    On {
        /// Relay number 0 to 7
        relay: u8,
    },

    /// Set a relay to off
    Off {
        /// Relay number 0 to 7
        relay: u8,
    },

    /// Toggle a relay
    Toggle {
        /// Relay number 0 to 7
        relay: u8,
    },

    /// Set all relays to on
    AllOn,

    /// Set all relays to off
    AllOff,

    /// Turn relay to on an all others to off
    Latch {
        /// Relay number 0 to 7
        relay: u8,
    },

    /// Turn relay on for 1 second
    Momentary {
        /// Relay number 0 to 7
        relay: u8,
    },

    /// Turn relay on for delay seconds
    Delay {
        /// Relay number 0 to 7
        relay: u8,
        /// Delay in seconds from 0 to 255
        delay: u8,
    },

    /// Queries the current RS485 address, this message is broadcasted.
    /// Only one module can be connected to the RS485 bus, more than one will be wrong!
    QueryAddress,

    /// Set the RS485 address
    SetAddress {
        /// The RS485 address can be from 1 to 247
        #[arg(value_parser=maybe_hex::<u8>)]
        address: u8,
    },
}

const fn about_text() -> &'static str {
    "8 channel relay controller for the command line"
}

#[derive(Parser, Debug)]
#[command(version, about=about_text(), long_about = None)]
struct CliArgs {
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

    // Connection type
    #[command(subcommand)]
    pub connection: CliConnection,

    /// Modbus timeout in milliseconds
    #[arg(long, default_value_t = 200)]
    timeout: u16,
}

fn logging_init(loglevel: LevelFilter) -> LoggerHandle {
    let log_handle = Logger::try_with_env_or_str(loglevel.as_str())
        .expect("Cannot init logging")
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
            .map(String::deref);
        let cause = cause.unwrap_or_else(|| {
            panic_info
                .payload()
                .downcast_ref::<&str>()
                .copied()
                .unwrap_or("<cause unknown>")
        });

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

    let (mut d, command) = match &args.connection {
        CliConnection::Tcp { address, command } => {
            let socket_addr = address
                .parse()
                .with_context(|| format!("Cannot parse address {}", address))?;
            trace!("Open TCP address {}", socket_addr);
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
                println!("Use this command only if ONLY ONE module is connected to the RS485 bus!");
                let confirmation = Confirm::new()
                    .with_prompt("Do you want to continue?")
                    .default(false)
                    .show_default(true)
                    .interact()?;
                if !confirmation {
                    return Ok(());
                }
                if *address != proto::READ_ADDRESS_BROADCAST_ADDRESS {
                    info!(
                        "Ignore address {:#04x} use broadcast address {:#04x}",
                        address,
                        proto::READ_ADDRESS_BROADCAST_ADDRESS
                    );
                }
                proto::READ_ADDRESS_BROADCAST_ADDRESS
            } else {
                *address
            };
            trace!("Open RTU {} address {:#04x}", device, address);
            (
                R413D08::new(
                    tokio_modbus::client::sync::rtu::connect_slave(
                        &r413d08_lib::tokio_serial::serial_port_builder(device),
                        tokio_modbus::Slave(address),
                    )
                    .with_context(|| format!("Cannot open device {}", device))?,
                ),
                command,
            )
        }
    };
    d.set_timeout(Duration::from_millis(args.timeout as u64));

    match command {
        CliCommands::Status => {
            let rsp = d.read_ports().with_context(|| "Cannot read status")?;
            println!("Status:");
            for (idx, relay) in rsp.iter().enumerate() {
                println!(
                    "  Relay {}: {}",
                    idx,
                    if *relay == r413d08_lib::State::Close {
                        "OFF"
                    } else {
                        "ON"
                    }
                );
            }
        }
        CliCommands::On { relay } => {
            d.set_port_open(*relay)?;
        }
        CliCommands::AllOn => d.set_all_open()?,
        CliCommands::Off { relay } => {
            d.set_port_close(*relay)?;
        }
        CliCommands::AllOff => d.set_all_close()?,
        CliCommands::Toggle { relay } => {
            d.set_port_toggle(*relay)?;
        }
        CliCommands::Latch { relay } => d.set_port_latch(*relay)?,
        CliCommands::Momentary { relay } => d.set_port_momentary(*relay)?,
        CliCommands::Delay { relay, delay } => d.set_port_delay(*relay, *delay)?,
        CliCommands::QueryAddress => {
            let rsp = d
                .read_address()
                .with_context(|| "Cannot read RS485 address")?;
            println!("RS485 address: {:#04x}", rsp);
        }
        CliCommands::SetAddress { address } => {
            d.set_address(*address)
                .with_context(|| "Cannot set RS485 address")?;
        }
    }

    Ok(())
}
