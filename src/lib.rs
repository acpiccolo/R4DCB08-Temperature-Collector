#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
# R4DCB08 Modbus Library

A Rust library for interacting with the R4DCB08 8-channel temperature data acquisition module
via the Modbus RTU and TCP protocols.

This library provides a high-level API to abstract the underlying Modbus communication,
allowing you to easily read temperature data and configure device parameters. It is built
on top of the powerful `tokio-modbus` crate and leverages `tokio` for asynchronous
operations.

## Key Features

- **High-Level API**: Simplifies interaction with the R4DCB08 module.
- **Protocol Abstraction**: Handles the details of Modbus register mapping, data encoding,
  and decoding as defined in the `protocol` module.
- **Sync and Async Clients**: Offers both synchronous and asynchronous clients to fit
  different application needs.
- **RTU and TCP Support**: Communicate via serial (RTU) or network (TCP).
- **Feature-Gated**: Compile only the features you need, keeping the library lean.

## Usage Example

The following example demonstrates how to use the synchronous TCP client to connect to an
R4DCB08 module and read the temperatures from all 8 channels.

First, add the following to your `Cargo.toml`:
```toml
[dependencies]
r4dcb08_lib = { version = "0.2", features = ["tokio-tcp-sync"] }
tokio = { version = "1", features = ["full"] }
```

Then, use the client in your code:

```no_run
use r4dcb08_lib::tokio_sync_client::R4DCB08;
use std::net::SocketAddr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The IP address and port of the R4DCB08 module
    let socket_addr: SocketAddr = "192.168.1.100:502".parse()?;

    // Connect to the Modbus TCP device
    let mut modbus_ctx = tokio_modbus::client::sync::tcp::connect(socket_addr)?;

    // Create a new R4DCB08 client
    let mut client = R4DCB08::new(modbus_ctx);

    // Read temperatures from all 8 channels
    match client.read_temperatures() {
        Ok(temperatures) => {
            println!("Successfully read temperatures:");
            for (i, temp) in temperatures.iter().enumerate() {
                // The Temperature type implements Display for easy printing
                println!("  Channel {}: {}", i, temp);
            }
        }
        Err(e) => {
            eprintln!("Failed to read temperatures: {}", e);
        }
    }

    Ok(())
}
```

## Features

This crate uses feature flags to control which client implementations are included.
This helps to minimize the number of dependencies and reduce compile times.

| Feature          | Description                                                              |
|------------------|--------------------------------------------------------------------------|
| `tokio-rtu-sync` | Enables the synchronous client for Modbus RTU (serial) communication.    |
| `tokio-tcp-sync` | Enables the synchronous client for Modbus TCP (network) communication.   |
| `tokio-rtu`      | Enables the asynchronous client for Modbus RTU (serial) communication.   |
| `tokio-tcp`      | Enables the asynchronous client for Modbus TCP (network) communication.  |
| `serde`          | Enables serialization/deserialization for `protocol` data structures via `serde`. |

By default, `tokio-rtu-sync` and `tokio-rtu` are enabled.
*/

/// Defines data structures, constants, and protocol-level logic for the R4DCB08 module.
pub mod protocol;

#[cfg(any(
    feature = "tokio-rtu-sync",
    feature = "tokio-tcp-sync",
    feature = "tokio-rtu",
    feature = "tokio-tcp"
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "tokio-rtu-sync",
        feature = "tokio-tcp-sync",
        feature = "tokio-rtu",
        feature = "tokio-tcp"
    )))
)]
/// Common error types and utilities for Tokio-based clients.
pub mod tokio_common;

#[cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync")))
)]
/// Provides a synchronous, high-level client for interacting with the R4DCB08 module.
pub mod tokio_sync_client;

#[cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))))]
/// Provides an asynchronous, high-level client for interacting with the R4DCB08 module.
pub mod tokio_async_client;
