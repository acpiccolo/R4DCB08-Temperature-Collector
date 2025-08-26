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
- [Library Usage](#library-usage)
- [Command-Line Usage](#command-line-usage)
- [Cargo Features](#cargo-features)
- [License](#license)

## Hardware Requirements
To use this tool, you need:
- One or more **R4DCB08 Temperature Collectors**.
- Up to **8 DS18B20** Temperature Sensors.
- A **USB-to-RS485 converter** (for RTU mode).

![R4DCB08 Temperature Collector](/images/r4dcb08.png)

## Technical Documentation
For more detailed information, please refer to the official datasheets available in the [`docs/`](./docs/) directory:
- [`R4DCB08_description.pdf`](./docs/R4DCB08_description.pdf)
- [`R4DCB08_modbus_rtu_protocol.pdf`](./docs/R4DCB08_modbus_rtu_protocol.pdf)

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

## Library Usage

This project can also be used as a library in your own Rust applications. It provides a high-level, thread-safe `SafeClient` for easy interaction with the R4DCB08 module, available in both synchronous and asynchronous versions.

### Quick Start: Synchronous Client

Here's a quick example of how to use the synchronous `SafeClient` to read temperatures over a TCP connection.

#### Dependencies

First, add the required dependencies to your project:
```sh
cargo add R4DCB08@0.3 --no-default-features --features "tokio-tcp-sync,safe-client-sync,serde"
cargo add tokio-modbus@0.16
cargo add tokio@1 --features full
```

#### Example Usage

```rust
use r4dcb08_lib::{
    protocol::Address,
    tokio_sync_safe_client::SafeClient,
};
use tokio_modbus::client::sync::tcp;
use tokio_modbus::Slave;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the device and create a stateful, safe client
    let socket_addr = "192.168.1.100:502".parse()?;
    let ctx = tcp::connect_slave(socket_addr, Slave(*Address::default()))?;
    let mut client = SafeClient::new(ctx);

    // Use the client to interact with the device
    let temperatures = client.read_temperatures()?;

    println!("Successfully read temperatures. Current values: {}", temperatures);

    Ok(())
}
```

For asynchronous examples and low-level functions, please refer to the full library documentation.

## Command-Line Usage

This tool provides a range of commands for device discovery, configuration, and data acquisition.

### Connection Types
You can connect to the temperature collector via Modbus RTU (serial) or TCP.

- **RTU (Serial):**
  ```sh
  tempcol rtu --device /dev/ttyUSB0 --address 1 --baud-rate 9600 <COMMAND>
  ```
- **TCP:**
  ```sh
  tempcol tcp 192.168.0.222:502 <COMMAND>
  ```

### Global Options
These options can be used with any command:
- `--timeout <DURATION>`: Sets the I/O timeout for Modbus operations (e.g., `500ms`, `1s`). Default: `200ms`.
- `--delay <DURATION>`: Specifies a minimum delay between Modbus commands. Crucial for some USB-to-RS485 converters. Default: `50ms`.

### Available Commands

#### Help
To see a full list of commands and options:
```sh
tempcol --help
```

#### Device Discovery
- **Scan for RTU Devices:** Detects connected R4DCB08 devices on the specified serial port.
  ```sh
  tempcol rtu-scan --device /dev/ttyUSB0
  ```
- **Query RTU Address:** Finds the address of a single connected device.
  ```sh
  # Ensure only one device is connected on the bus
  tempcol rtu query-address
  ```

#### Read Commands
- **Read Temperatures:** Reads all 8 temperature sensor channels.
  ```sh
  tempcol tcp <IP:PORT> read
  ```
- **Read Temperature Corrections:** Shows the correction values for all channels.
  ```sh
  tempcol rtu --address 1 read-correction
  ```
- **Read Baud Rate:** Displays the device's configured baud rate.
  ```sh
  tempcol rtu --address 1 read-baud-rate
  ```
- **Read Auto-Report Interval:** Shows the automatic reporting interval.
  ```sh
  tempcol rtu --address 1 read-automatic-report
  ```
- **Read All:** Reads all primary device values at once.
  ```sh
  tempcol rtu --address 1 read-all
  ```

#### Set Commands
- **Set Temperature Correction:** Applies a calibration offset to a sensor.
  ```sh
  # Set channel 0 correction to -1.5°C
  tempcol rtu --address 1 set-correction 0 -1.5
  ```
- **Set Baud Rate:** Changes the device's baud rate (requires power cycle).
  ```sh
  tempcol rtu --address 1 set-baud-rate 19200
  ```
- **Set Device Address:** Assigns a new Modbus RTU address (1-247).
  ```sh
  tempcol rtu --address 1 set-address 2
  ```
- **Set Auto-Report Interval:** Configures the automatic reporting time (0 to disable).
  ```sh
  # Set to report every 30 seconds
  tempcol rtu --address 1 set-automatic-report 30
  ```

#### Daemon Mode
Run continuously to poll for temperatures.

- **Log to Console:**
  ```sh
  # Poll every 5 seconds and print to console
  tempcol rtu --address 1 daemon --poll-interval 5s console
  ```
- **Publish to MQTT:**
  ```sh
  # Poll and publish to an MQTT broker
  tempcol tcp <IP:PORT> daemon mqtt --mqtt-config /path/to/mqtt.yaml
  ```
  If `--mqtt-config` is omitted, `mqtt.yaml` is loaded from the current directory.

#### Other Commands
- **Factory Reset:** Resets the device to its default settings (requires power cycle).
  ```sh
  # Warning: This is irreversible
  tempcol rtu --address 1 factory-reset
  ```

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

This crate uses a feature-based system to minimize dependencies. When using it as a library, you should disable default features and select only the components you need.

- **`default`**: Enables `bin-dependencies`, intended for compiling the `tempcol` command-line tool.

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
- **`bin-dependencies`**: All features required to build the `tempcol` binary.

## License
Licensed under either of:
* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [Apache](http://www.apache.org/licenses/LICENSE-2.0))
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [MIT](http://opensource.org/licenses/MIT))

at your option.
