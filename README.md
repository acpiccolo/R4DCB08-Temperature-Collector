<p align="center">
  <a href="https://github.com/acpiccolo/R4DCB08-Temperature-Collector/actions"
    ><img
      src="https://github.com/acpiccolo/R4DCB08-Temperature-Collector/actions/workflows/check.yml/badge.svg?branch=main"
      alt="GitHub Actions workflow status"
  /></a>
  <a href="https://conventionalcommits.org"
    ><img
      src="https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg"
      alt="Conventional commits"
  /></a>
  <a href="https://github.com/acpiccolo/R4DCB08-Temperature-Collector/blob/main/LICENSE-MIT"
    ><img
      src="https://img.shields.io/github/license/acpiccolo/R4DCB08-Temperature-Collector"
      alt="Repository license"
  /></a>
  <a href="https://github.com/acpiccolo/R4DCB08-Temperature-Collector/blob/main/LICENSE-APACHE"
    ><img
      src="https://img.shields.io/github/license/acpiccolo/R4DCB08-Temperature-Collector"
      alt="Repository license"
  /></a>
</p>

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
