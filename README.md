[![CI](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/actions/workflows/check.yml/badge.svg)](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/actions/workflows/check.yml)
[![dependency status](https://deps.rs/repo/github/acpiccolo/R4DCB08-Temperature-Collector/status.svg)](https://deps.rs/repo/github/acpiccolo/R4DCB08-Temperature-Collector)
[![CI](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/blob/main/LICENSE-MIT)
[![CI](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://github.com/acpiccolo/R4DCB08-Temperature-Collector/blob/main/LICENSE-APACHE)
[![CI](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)

# R4DCB08 temperature collector
This RUST project can read and write a R4DCB08 temperature collector/monitor from the command line.

## Hardware
The following hardware is required for this project:
* One or more R4DCB08 temperature collectors.
* One or more DS18B20 temperature sensors.
* One USB-RS485 converter.

![R4DCB08 temperature collector](/images/r4dcb08.png)

### Data sheet R4DCB08 temperature collector
* Operating Voltage: DC 6-24 Volt
* Operating Current: 8-13 Milli-Ampere (depending on the connected DS18B20)
* Up to 8 DS18B20 temperature sensors can be connected
* Measurable temperature range -55°C to +125°C
* 0.5° accuracy from -10°C to +85°C

## Compilation
1. Install Rust e.g. using [these instructions](https://www.rust-lang.org/learn/get-started).
2. Ensure that you have a C compiler and linker.
3. Clone `git clone https://github.com/acpiccolo/R4DCB08-Temperature-Collector.git`
4. Run `cargo install --path .` to install the binary. Alternatively,
   check out the repository and run `cargo build --release`. This will compile
   the binary to `target/release/tempcol`.

## Getting started
To see all available commands:
```
tempcol --help
```
For RTU Modbus connected temperature collectors:
```
tempcol rtu-scan
tempcol rtu --address 1 --baudrate 9600 read
```
For TCP Modbus connected temperature collectors:
```
tempcol tcp 192.168.0.222:502 read
```
You can even use this tool as a daemon for a MQTT broker:
```
tempcol rtu --address 1 --baudrate 9600 deamon mqtt --username my_name --password my_secret mqtt://localhost:1883
```

### Cargo Features
| Feature | Purpose | Default |
| :--- | :------ | :-----: |
| `tokio-rtu-sync` | Enable the implementation for the tokio modbus synchronous RTU client | ✅ |
| `tokio-rtu` | Enable the implementation for the tokio modbus asynchronous RTU client | ✅ |
| `tokio-tcp-sync` | Enable the implementation for the tokio modbus synchronous TCP client | - |
| `tokio-tcp` | Enable the implementation for the tokio modbus asynchronous TCP client | - |
| `bin-dependencies` | Enable all features required by the binary | ✅ |


## License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
