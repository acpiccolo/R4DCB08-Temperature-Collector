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
To continuously collect data and publish it to an MQTT broker, you will need to create an MQTT configuration file (see below).
Example:
```sh
tempcol rtu --address 1 --baudrate 9600 daemon mqtt --mqtt-config /path/to/your/mqtt.yaml
```
If the `--mqtt-config` option is omitted, the application will automatically look for a file named `mqtt.yaml` in the current working directory.

### MQTT Configuration File

All MQTT connection parameters are configured exclusively through a YAML file. This approach centralizes settings like the broker URL, credentials, topic, QoS, and client ID.

**Path Specification:**
- Use the `--mqtt-config <PATH>` option to specify the path to your MQTT configuration file. For example:
  ```sh
  tempcol tcp 192.168.0.222:502 daemon mqtt --mqtt-config /path/to/your/mqtt.yaml
  ```
- If the `--mqtt-config <PATH>` option is **not** provided, the application will attempt to load `mqtt.yaml` from the current working directory.

If the specified configuration file (either via option or the default `mqtt.yaml`) is not found, cannot be read, or is improperly formatted, the program will exit with an error.

**Mandatory Fields:**
- The `uri` field within the MQTT configuration YAML file is **mandatory**.

**Configuration Details:**

The YAML file allows you to set the following parameters:
- `uri` (String, Mandatory): URI of the MQTT broker (e.g., "tcp://localhost:1883").
- `username` (String, Optional): Username for MQTT broker authentication.
- `password` (String, Optional): Password for MQTT broker authentication.
- `topic` (String, Optional): Base MQTT topic. Defaults to "r4dcb08" if not set.
- `qos` (Integer, Optional): MQTT Quality of Service level (0, 1, or 2). Defaults to 0 if not set.
- `client_id` (String, Optional): Client ID for the MQTT connection. If not provided, a random ID is generated by the MQTT client library.

**Example `mqtt.yaml`:**

```yaml
# Sample MQTT Configuration for R4DCB08 Temperature Collector CLI

# URI of the MQTT broker (Mandatory).
# Example: "tcp://localhost:1883" or "mqtts://secure-broker.com:8883"
uri: "tcp://localhost:1883"

# Username for MQTT broker authentication (optional).
# username: "your_username"

# Password for MQTT broker authentication (optional).
# password: "your_password"

# Base MQTT topic to publish temperature readings to.
# Readings for each channel will be published to "{topic}/CH{channel_index}".
# Example: "r4dcb08/CH0", "r4dcb08/CH1", etc.
# Defaults to "r4dcb08".
topic: "r4dcb08/house/temperature"

# MQTT Quality of Service (QoS) level for publishing messages.
# 0: At most once, 1: At least once, 2: Exactly once.
# Defaults to 0.
qos: 1

# Client ID for the MQTT connection (optional).
# If not provided, a random ID will be generated by the client library.
# client_id: "r4dcb08-collector-main"
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
