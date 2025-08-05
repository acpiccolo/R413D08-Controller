use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use r413d08_lib::protocol as proto;
use std::time::Duration;

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
pub enum CliConnection {
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
pub struct CliArgs {
    /// Verbosity level (-v, -vv, -vvv).
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    /// Connection type (TCP or RTU) and associated command.
    #[command(subcommand)]
    pub connection: CliConnection,

    /// Modbus I/O timeout duration (e.g., "200ms", "1s").
    #[arg(value_parser = humantime::parse_duration, long, default_value = "200ms")]
    pub timeout: Duration,
}
