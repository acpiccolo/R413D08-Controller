//! A command-line interface (CLI) application for controlling an R413D08
//! 8-channel relay module via Modbus RTU (Serial) or Modbus TCP.
//!
//! This tool allows reading relay statuses, controlling individual or all relays
//! using various modes (On, Off, Toggle, Latch, Momentary, Delay), and managing
//! the device's Modbus address. It uses the `r413d08_lib` crate which provides
//! the necessary protocol definitions and a synchronous Modbus client.

use anyhow::{Context, Result};
use clap::Parser;
use dialoguer::Confirm;
use flexi_logger::{Logger, LoggerHandle};
use log::*;
use r413d08_lib::{protocol as proto, tokio_sync_client::R413D08};
use std::{ops::Deref, panic};

mod commandline;

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
    let args = commandline::CliArgs::parse();

    let _log_handle = logging_init(args.verbose.log_level_filter());

    let (mut client, command) = match &args.connection {
        commandline::CliConnection::Tcp { address, command } => {
            let socket_addr = address
                .parse()
                .with_context(|| format!("Cannot parse TCP address '{address}'"))?;
            trace!("Connecting via TCP to {socket_addr}...");
            (
                R413D08::new(
                    tokio_modbus::client::sync::tcp::connect(socket_addr)
                        .with_context(|| format!("Cannot open {socket_addr:?}"))?,
                ),
                command,
            )
        }
        commandline::CliConnection::Rtu {
            device,
            address,
            command,
        } => {
            let address = if command == &commandline::CliCommands::QueryAddress {
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
                    info!("Ignore address {address} use broadcast address {broadcast_address}");
                }
                broadcast_address
            } else {
                *address
            };
            trace!("Connecting via RTU to {device} address {address}");
            (
                R413D08::new(
                    tokio_modbus::client::sync::rtu::connect_slave(
                        &r413d08_lib::tokio_serial::serial_port_builder(device),
                        tokio_modbus::Slave(*address),
                    )
                    .with_context(|| format!("Cannot open RTU device {device}"))?,
                ),
                command,
            )
        }
    };
    client.set_timeout(Some(args.timeout));

    match command {
        commandline::CliCommands::Status => {
            let rsp = client.read_ports().context("Failed to read port status")?;
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
        commandline::CliCommands::On { relay } => {
            client
                .set_port_open(*relay)
                .with_context(|| format!("Failed to turn ON relay {}", **relay))?;
            println!("Relay {} turned ON", **relay);
        }
        commandline::CliCommands::AllOn => {
            client
                .set_all_open()
                .context("Failed to turn ALL relays ON")?;
            println!("All relays turned ON");
        }
        commandline::CliCommands::Off { relay } => {
            client
                .set_port_close(*relay)
                .with_context(|| format!("Failed to turn OFF relay {}", **relay))?;
            println!("Relay {} turned OFF", **relay);
        }
        commandline::CliCommands::AllOff => {
            client
                .set_all_close()
                .context("Failed to turn ALL relays OFF")?;
            println!("All relays turned OFF");
        }
        commandline::CliCommands::Toggle { relay } => {
            client
                .set_port_toggle(*relay)
                .with_context(|| format!("Failed to toggle relay {}", **relay))?;
            println!("Relay {} toggled", **relay);
        }
        commandline::CliCommands::Latch { relay } => {
            client
                .set_port_latch(*relay)
                .with_context(|| format!("Failed to latch relay {}", **relay))?;
            println!("Relay {} latched ON (others OFF)", **relay);
        }
        commandline::CliCommands::Momentary { relay } => {
            client
                .set_port_momentary(*relay)
                .with_context(|| format!("Failed to activate momentary relay {}", **relay))?;
            println!("Relay {} activated momentarily", **relay);
        }
        commandline::CliCommands::Delay { relay, delay } => {
            client
                .set_port_delay(*relay, *delay)
                .with_context(|| format!("Failed to set delay for relay {}", **relay))?;
            println!(
                "Relay {} activated with {} second delay before turning OFF",
                **relay, delay
            );
        }
        commandline::CliCommands::QueryAddress => {
            // Note: Connection was already set up with broadcast address above
            let address = client
                .read_address()
                .context("Failed to query device address (ensure only one device is connected)")?;
            println!("Device responded with address: {address}");
        }
        commandline::CliCommands::SetAddress { address } => {
            client
                .set_address(*address)
                .with_context(|| format!("Failed to set new Modbus address to {address}"))?;
            println!(
                "Successfully sent command to set Modbus address to {address}. \
                 Remember to use this new address for future communication."
            );
        }
    }

    Ok(())
}
