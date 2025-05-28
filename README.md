[![CI](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/actions/workflows/check.yml/badge.svg)](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/actions/workflows/check.yml)
[![dependency status](https://deps.rs/repo/github/acpiccolo/R4DCB08-Temperature-Collector/status.svg)](https://deps.rs/repo/github/acpiccolo/R4DCB08-Temperature-Collector)
[![CI](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/blob/main/LICENSE-MIT)
[![CI](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/blob/main/LICENSE-APACHE)
[![CI](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)

# R4DCB08 Temperature Collector

This Rust project enables communication with an **R4DCB08 temperature collector** using **Modbus RTU/TCP** from the command line.

## Table of Contents
- [Hardware Requirements](#hardware-requirements)
- [Technical Specifications](#technical-specifications-r4dcb08)
- [Installation & Compilation](#installation--compilation)
- [Usage](#usage)
- [Cargo Features](#cargo-features)
- [License](#license)

## Hardware Requirements
To use this tool, you need:
- One or more **R4DCB08 Temperature Collectors**.
- Up to **8 DS18B20** Temperature Sensors.
- A **USB-to-RS485 converter** (for RTU mode).

![R4DCB08 Temperature Collector](/images/r4dcb08.png)

## Technical Specifications R4DCB08
| Feature | Details |
|---------|---------|
| **Operating Voltage** | 6-24V DC |
| **Operating Current** | 8-13mA (depends on connected DS18B20 sensors) |
| **Temperature Range** | -55°C to +125°C |
| **Accuracy** | ±0.5°C from -10°C to +85°C |
| **Baud Rates** | 1200, 2400, 4800, 9600 (default), 19200 |
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
   git clone https://github.com/acpiccolo/R4DCB08-Temperature-Collector.git
   cd R4DCB08-Temperature-Collector
   ```
2. **Compile the project**:
   ```sh
   cargo build --release
   ```
   The compiled binary will be available at:
   ```sh
   target/release/tempcol
   ```
3. **(Optional) Install the binary system-wide**:
   ```sh
   cargo install --path .
   ```
   This installs `tempcol` to `$HOME/.cargo/bin`, making it accessible from anywhere.

## Usage
### View Available Commands
To list all available commands and their options, run:
```sh
tempcol --help
```
### Scan for Connected RTU Devices
To detect available Modbus RTU devices:
```sh
tempcol rtu-scan
```
### Read Temperatures Values
For **RTU Modbus (RS485) connected** devices:
```sh
tempcol rtu --address 1 --baudrate 9600 read
```
For **TCP Modbus connected** devices:
```sh
tempcol tcp 192.168.0.222:502 read
```
#### Set Temperature Correction
To apply a temperature correction of **-1.5°C** to sensor channel 0
```sh
tempcol rtu --address 1 --baudrate 9600 set-correction 0 -1.5
```
#### Run as a Daemon for MQTT Integration
To continuously collect data and publish it to an MQTT broker:
```sh
tempcol rtu --address 1 --baudrate 9600 daemon mqtt --username my_name --password my_secret mqtt://localhost:1883
```

## Cargo Features
| Feature | Purpose | Default |
| :--- | :------ | :-----: |
| `tokio-rtu-sync` | Enable support for synchronous tokio RTU client | ✅ |
| `tokio-rtu` | Enable support for asynchronous tokio RTU client | ✅ |
| `tokio-tcp-sync` | Enable support synchronous tokio TCP client | - |
| `tokio-tcp` | Enable support asynchronous tokio TCP client | - |
| `bin-dependencies` | Enable all features required by the binary | ✅ |
| `serde` | Enable the serde framework for protocol structures | - |

## License
Licensed under either of:
* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [Apache](http://www.apache.org/licenses/LICENSE-2.0))
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [MIT](http://opensource.org/licenses/MIT))

at your option.
