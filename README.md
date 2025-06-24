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

### Cargo Features
| Feature | Purpose | Default |
| :--- | :------ | :-----: |
| `tokio-rtu-sync` | Enable the implementation for the tokio modbus synchronous RTU client | ✅ |
| `tokio-rtu` | Enable the implementation for the tokio modbus asynchronous RTU client | ✅ |
| `tokio-tcp-sync` | Enable the implementation for the tokio modbus synchronous TCP client | - |
| `tokio-tcp` | Enable the implementation for the tokio modbus asynchronous TCP client | - |
| `bin-dependencies` | Enable all features required by the binary | ✅ |
| `serde` | Enable the serde framework for protocol structures | - |


## License
Licensed under either of:
* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [Apache](http://www.apache.org/licenses/LICENSE-2.0))
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [MIT](http://opensource.org/licenses/MIT))

at your option.
