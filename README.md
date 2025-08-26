[![CI](https://github.com/acpiccolo/R413D08-Controller/actions/workflows/check.yml/badge.svg)](https://github.com/acpiccolo/R413D08-Controller/actions/workflows/check.yml)
[![dependency status](https://deps.rs/repo/github/acpiccolo/R413D08-Controller/status.svg)](https://deps.rs/repo/github/acpiccolo/R413D08-Controller)
[![CI](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/acpiccolo/R413D08-Controller/blob/main/LICENSE-MIT)
[![CI](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://github.com/acpiccolo/R413D08-Controller/blob/main/LICENSE-APACHE)
[![CI](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)

# R413D08 8 Channel Module
This Rust project enables communication with an **R413D08 8 Channel Module** using **Modbus RTU/TCP** from the command line. It should also work with the R421A08 relay card because it has the same Modbus commands.

## Table of Contents
- [Hardware Requirements](#hardware-requirements)
- [Technical Documentation](#technical-documentation)
- [Technical Specifications](#technical-specifications-r413d08)
- [Installation & Compilation](#installation--compilation)
- [Command-Line Usage](#command-line-usage)
- [Library Usage](#library-usage)
- [Cargo Features](#cargo-features)
- [License](#license)

## Hardware Requirements
To use this tool, you need:
- One or more **R413D08 8 Channel Module**.
- One or more **1-8 Channel Relay Board**.
- A **USB-to-RS485 converter** (for RTU mode).

![R413D08 8 Channel Module](/images/r413d08.png)

## Technical Documentation
For more detailed information, please refer to the official datasheets available in the [`docs/`](./docs/) directory:
- [`8.Channel.Multifunction.RS485.Module.command.pdf`](./docs/8.Channel.Multifunction.RS485.Module.command.pdf)

## Technical Specifications R413D08
| Feature | Details |
|---------|---------|
| **Operating Voltage** | 5V DC (5V version) or 6-24V DC (12V version) |
| **Operating Current** | 10-15mA |
| **Baud Rates** | 9600 |
| **Data Format** | N, 8, 1 (No parity, 8 data bits, 1 stop bit) |
| **Communication Protocol** | Modbus RTU/TCP |

## Installation & Compilation

### Prerequisites
Ensure you have the following dependencies installed before proceeding:
- **Rust and Cargo**: Install via [rustup](https://rustup.rs/)
- **Git**: To clone the repository

### **Building from Source**
1. **Clone the repository**:
   ```sh
   git clone https://github.com/acpiccolo/R413D08-Controller.git
   cd R413D08-Controller
   ```
2. **Compile the project**:
   ```sh
   cargo build --release
   ```
   The compiled binary will be available at:
   ```sh
   target/release/relay
   ```
3. **(Optional) Install the binary system-wide**:
   ```sh
   cargo install --path .
   ```
   This installs `relay` to `$HOME/.cargo/bin`, making it accessible from anywhere.

## Command-Line Usage

This tool provides a range of commands for device discovery, configuration, and data acquisition.

### Connection Types
You can connect to the relay module via Modbus RTU (serial) or TCP.

- **RTU (Serial):**
  ```sh
  relay rtu --address 1 <COMMAND>
  ```
- **TCP:**
  ```sh
  relay tcp 192.168.0.222:502 <COMMAND>
  ```

### Available Commands

#### Help
To see a full list of commands and options:
```sh
relay --help
```

#### Read Commands
- **Read Relay Status:** Reads the ON/OFF status of all 8 relays.
  ```sh
  relay tcp 192.168.0.222:502 status
  ```

#### Set Commands
- **Turn a Relay ON:**
  ```sh
  # Turn on relay 0
  relay rtu --address 1 on 0
  ```
- **Turn a Relay OFF:**
  ```sh
  # Turn off relay 3
  relay rtu --address 1 off 3
  ```

## Library Usage

This project can also be used as a library in your own Rust applications. It provides a high-level, thread-safe `SafeClient` for easy interaction with the R413D08 module, available in both synchronous and asynchronous versions.

### Quick Start: Synchronous Client

Here's a quick example of how to use the synchronous `SafeClient` to read relay statuses over a TCP connection.

#### Dependencies

First, add the required dependencies to your project:
```sh
cargo add R413D08@0.3 --no-default-features --features "tokio-tcp-sync,safe-client-sync,serde"
cargo add tokio-modbus@0.16
cargo add tokio@1 --features full
```

#### Example Usage

```rust,no_run
use r413d08_lib::{
    protocol::{Address, Port},
    tokio_sync_safe_client::SafeClient,
};
use tokio_modbus::client::sync::tcp;
use tokio_modbus::Slave;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the device and create a stateful, safe client
    let socket_addr = "192.168.1.100:502".parse()?;
    let ctx = tcp::connect_slave(socket_addr, Slave(*Address::default()))?;
    let client = SafeClient::new(ctx);

    // Use the client to interact with the device
    client.set_port_open(Port::try_from(0)?)?;
    let status = client.read_ports()?;

    println!("Successfully turned on relay 0. Current status: {}", status);

    Ok(())
}
```

For more advanced use cases, the library also provides low-level, stateless functions in the `r413d08_lib::tokio_sync` and `r413d08_lib::tokio_async` modules.

## Cargo Features

This crate uses a feature-based system to minimize dependencies. When using it as a library, you should disable default features and select only the components you need.

- **`default`**: Enables `bin-dependencies`, intended for compiling the `relay` command-line tool.

### Client Features
- **`tokio-rtu-sync`**: Synchronous (blocking) RTU client.
- **`tokio-tcp-sync`**: Synchronous (blocking) TCP client.
- **`tokio-rtu`**: Asynchronous (non-blocking) RTU client.
- **`tokio-tcp`**: Asynchronous (non-blocking) TCP client.

### High-Level Wrappers
- **`safe-client-sync`**: A thread-safe, stateful wrapper for synchronous clients.
- **`safe-client-async`**: A thread-safe, stateful wrapper for asynchronous clients.

### Utility Features
- **`serde`**: Implements `serde::Serialize` and `serde::Deserialize` for protocol structs.
- **`bin-dependencies`**: All features required to build the `relay` binary.



## License
Licensed under either of:
* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [Apache](http://www.apache.org/licenses/LICENSE-2.0))
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [MIT](http://opensource.org/licenses/MIT))

at your option.
