//! A library for controlling the R413D08 8-channel relay module via Modbus.
//!
//! This crate provides two main ways to interact with the R413D08 relay module:
//!
//! 1.  **High-Level, Safe Clients**: Stateful, thread-safe clients that are easy to share and use in concurrent applications. This is the recommended approach for most users. See [`tokio_sync_safe_client::SafeClient`] (blocking) and [`tokio_async_safe_client::SafeClient`] (`async`).
//!
//! 2.  **Low-Level, Stateless Functions**: A set of stateless functions that
//!     directly map to the device's Modbus commands. This API offers maximum
//!     flexibility but requires manual management of the Modbus context. See
//!     the [`tokio_sync`] and [`tokio_async`] modules.
//!
//! ## Features
//!
//! - **Protocol Implementation**: Complete implementation of the R413D08 Modbus protocol.
//! - **Stateful, Thread-Safe Clients**: For easy and safe concurrent use.
//! - **Stateless, Low-Level Functions**: For maximum flexibility and control.
//! - **Synchronous and Asynchronous APIs**: Both blocking and `async/await` APIs are available.
//! - **Strongly-Typed API**: Utilizes Rust's type system for protocol correctness (e.g., `Port`, `Address`, `PortState`).
//!
//! ## Quick Start
//!
//! This example shows how to use the recommended high-level, synchronous `SafeClient`.
//!
//! ```no_run
//! use r4dcb08_lib::{
//!     protocol::Address,
//!     tokio_sync_safe_client::SafeClient,
//! };
//! use tokio_modbus::client::sync::tcp;
//! use tokio_modbus::Slave;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to the device and create a stateful, safe client
//!     let socket_addr = "192.168.1.100:502".parse()?;
//!     let ctx = tcp::connect_slave(socket_addr, Slave(*Address::default()))?;
//!     let mut client = SafeClient::new(ctx);
//!
//!     // Use the client to interact with the device
//!     let temperatures = client.read_temperatures()?;
//!
//!     println!("Successfully read temperatures. Current values: {}", temperatures);
//!
//!     Ok(())
//! }
//! ```
//!
//! For more details, see the documentation for the specific client you wish to use.

pub mod protocol;

#[cfg(any(
    feature = "tokio-rtu-sync",
    feature = "tokio-tcp-sync",
    feature = "tokio-rtu",
    feature = "tokio-tcp"
))]
pub mod tokio_common;

#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync")))
)]
#[cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync"))]
pub mod tokio_sync;

#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))))]
#[cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))]
pub mod tokio_async;

#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync")))
)]
#[cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync"))]
pub mod tokio_sync_safe_client;

#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))))]
#[cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))]
pub mod tokio_async_safe_client;
