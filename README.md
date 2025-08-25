[![CI](https://github.com/acpiccolo/R413D08-Controller/actions/workflows/check.yml/badge.svg)](https://github.com/acpiccolo/R413D08-Controller/actions/workflows/check.yml)
[![dependency status](https://deps.rs/repo/github/acpiccolo/R413D08-Controller/status.svg)](https://deps.rs/repo/github/acpiccolo/R413D08-Controller)
[![CI](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/acpiccolo/R413D08-Controller/blob/main/LICENSE-MIT)
[![CI](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://github.com/acpiccolo/R413D08-Controller/blob/main/LICENSE-APACHE)
[![CI](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)

# R413D08 8 Channel Module
This Rust project enables communication with an **R413D08 8 Channel Module** using **Modbus RTU/TCP** from the command line. It should also work with the R421A08 relay card because it has the same Modbus commands.

## Table of Contents
- [Hardware Requirements](#hardware-requirements)
- [Technical Specifications](#technical-specifications-r4dcb08)
- [Installation & Compilation](#installation--compilation)
- [Usage](#usage)
- [Cargo Features](#cargo-features)
- [License](#license)

## Hardware Requirements
To use this tool, you need:
- One or more **R413D08 8 Channel Module**.
- One or more **1-8 Channel Relay Board**.
- A **USB-to-RS485 converter** (for RTU mode).

![R413D08 8 Channel Module](/images/r413d08.png)

## Technical Specifications R4DCB08
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

## Usage
### View Available Commands
To list all available commands and their options, run:
```sh
relay --help
```
### Read relay status values
For **RTU Modbus (RS485) connected** devices:
```sh
relay rtu --address 1 status
```
For **TCP Modbus connected** devices:
```sh
relay tcp 192.168.0.222:502 status
```
#### Set relay '0' to 'On'
For RTU Modbus:
```sh
relay rtu --address 1 on 0
```
For TCP Modbus:
```sh
relay tcp 192.168.0.222:502 on 0
```
#### Turn Off Relay '3'
For RTU Modbus:
```sh
relay rtu --address 1 off 3
```
For TCP Modbus:
```sh
relay tcp 192.168.0.222:502 off 3
```

## Library Usage

This crate can also be used as a library to control R413D08 modules from your own Rust application.

The recommended way is to use the high-level, thread-safe `SafeClient`.

### Library Example

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

# Cargo Features

This crate uses a feature-based system to allow you to select the specific components you need, minimizing dependencies and compile times.

### For Binary Users

If you are using the `relay` command-line tool, no action is needed. The binary is compiled with the `default` feature, which automatically enables all necessary functionalities for both RTU and TCP communication.

### For Library Users

If you are using this project as a library, you can customize your build by enabling only the features you require. This is ideal for optimizing your application's footprint.

Below is a detailed breakdown of the available features:

| Feature             | Description                                                                                                                                  | Default for Library | Default for Binary |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------|:-------------------:|:------------------:|
| **`bin-dependencies`**  | Enables all features required to build the `relay` binary, including CLI parsing, logging, and both RTU/TCP clients.                      |                     |         ✅         |
| **`tokio-rtu-sync`**    | **Synchronous RTU Client:** Enables the `tokio-modbus` RTU client for synchronous (blocking) serial communication.                           |                     |         ✅         |
| **`tokio-rtu`**         | **Asynchronous RTU Client:** Enables the `tokio-modbus` RTU client for asynchronous (non-blocking) serial communication.                     |                     |                    |
| **`tokio-tcp-sync`**    | **Synchronous TCP Client:** Enables the `tokio-modbus` TCP client for synchronous (blocking) network communication.                          |                     |         ✅         |
| **`tokio-tcp`**         | **Asynchronous TCP Client:** Enables the `tokio-modbus` TCP client for asynchronous (non-blocking) network communication.                      |                     |                    |
| **`safe-client-sync`**  | **Stateful Synchronous Client:** Provides a thread-safe, stateful wrapper for easy synchronous interaction with the device.                 |                     |         ✅         |
| **`safe-client-async`** | **Stateful Asynchronous Client:** Provides a thread-safe, stateful wrapper for easy asynchronous interaction with the device.                |                     |                    |
| **`serde`**             | **Serialization:** Implements `serde::Serialize` and `serde::Deserialize` for all protocol-related structs, useful for data exchange.      |                     |         ✅         |



## License
Licensed under either of:
* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [Apache](http://www.apache.org/licenses/LICENSE-2.0))
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [MIT](http://opensource.org/licenses/MIT))

at your option.
