//! Asynchronous `tokio-modbus` client for the R4DCB08 temperature module.
//!
//! This module provides a high-level API (`SafeClient` struct) to interact with
//! the R4DCB08 8-channel temperature module using Modbus RTU or TCP. It handles
//! the conversion between Rust types defined in the `crate::protocol` module and
//! the raw Modbus register values.
//!
//! All client methods are `async` and must be `.await`ed.
//!
//! ## Example
//!
//! ```no_run
//! use r4dcb08_lib::{
//!     protocol::Address,
//!     tokio_async_safe_client::SafeClient,
//! };
//! use tokio_modbus::client::tcp;
//! use tokio_modbus::Slave;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to the device and create a stateful, safe client
//!     let socket_addr = "192.168.1.100:502".parse()?;
//!     let ctx = tcp::connect_slave(socket_addr, Slave(*Address::default())).await?;
//!     let client = SafeClient::new(ctx);
//!
//!     // Use the client to interact with the device
//!     let temperatures = client.read_temperatures().await?;
//!
//!     println!("Successfully read temperatures. Current values: {}", temperatures);
//!
//!     Ok(())
//! }
//! ```

use crate::{protocol as proto, tokio_async, tokio_common::Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_modbus::{client::Context, prelude::*};

/// Asynchronous client for interacting with the R4DCB08 temperature module over Modbus.
///
/// This struct provides methods to read sensor data and configure the module's
/// operational parameters by wrapping `tokio-modbus` asynchronous operations.
///
/// All methods that interact with the Modbus device are `async` and return `Future`s.
#[derive(Clone)]
pub struct SafeClient {
    ctx: Arc<Mutex<Context>>,
}

impl SafeClient {
    /// Creates a new `SafeClient` with a given `tokio-modbus` asynchronous context.
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx: Arc::new(Mutex::new(ctx)),
        }
    }

    /// Creates a new `SafeClient` from a shared `tokio-modbus` asynchronous context.
    pub fn from_shared(ctx: Arc<Mutex<Context>>) -> Self {
        Self { ctx }
    }

    /// Clones the shared `tokio-modbus` asynchronous context.
    pub fn clone_shared(&self) -> Arc<Mutex<Context>> {
        self.ctx.clone()
    }

    /// Reads the current temperatures from all 8 available channels in degrees Celsius (°C).
    pub async fn read_temperatures(&self) -> Result<proto::Temperatures> {
        let mut ctx = self.ctx.lock().await;
        tokio_async::R4DCB08::read_temperatures(&mut ctx).await
    }

    /// Reads the configured temperature correction values (°C) for all 8 channels.
    pub async fn read_temperature_correction(&self) -> Result<proto::TemperatureCorrection> {
        let mut ctx = self.ctx.lock().await;
        tokio_async::R4DCB08::read_temperature_correction(&mut ctx).await
    }

    /// Sets a temperature correction value for a specific channel.
    pub async fn set_temperature_correction(
        &self,
        channel: proto::Channel,
        correction: proto::Temperature,
    ) -> Result<()> {
        let mut ctx = self.ctx.lock().await;
        tokio_async::R4DCB08::set_temperature_correction(&mut ctx, channel, correction).await
    }

    /// Reads the automatic temperature reporting interval.
    pub async fn read_automatic_report(&self) -> Result<proto::AutomaticReport> {
        let mut ctx = self.ctx.lock().await;
        tokio_async::R4DCB08::read_automatic_report(&mut ctx).await
    }

    /// Sets the automatic temperature reporting interval.
    pub async fn set_automatic_report(&self, report: proto::AutomaticReport) -> Result<()> {
        let mut ctx = self.ctx.lock().await;
        tokio_async::R4DCB08::set_automatic_report(&mut ctx, report).await
    }

    /// Reads the current Modbus communication baud rate setting from the device.
    pub async fn read_baud_rate(&self) -> Result<proto::BaudRate> {
        let mut ctx = self.ctx.lock().await;
        tokio_async::R4DCB08::read_baud_rate(&mut ctx).await
    }

    /// Sets the Modbus communication baud rate for the device.
    pub async fn set_baud_rate(&self, baud_rate: proto::BaudRate) -> Result<()> {
        let mut ctx = self.ctx.lock().await;
        tokio_async::R4DCB08::set_baud_rate(&mut ctx, baud_rate).await
    }

    /// Resets the R4DCB08 module to its factory default settings.
    pub async fn factory_reset(&self) -> Result<()> {
        let mut ctx = self.ctx.lock().await;
        tokio_async::R4DCB08::factory_reset(&mut ctx).await
    }

    /// Reads the current Modbus device address (Slave ID) from the module.
    pub async fn read_address(&self) -> Result<proto::Address> {
        let mut ctx = self.ctx.lock().await;
        tokio_async::R4DCB08::read_address(&mut ctx).await
    }

    /// Sets a new Modbus device address.
    ///
    /// A successful call makes the existing `Context` invalid (as it
    /// still points to the old address). This function automatically
    /// updates the slave ID within its managed `Context`.
    pub async fn set_address(&self, new_address: proto::Address) -> Result<()> {
        let mut ctx = self.ctx.lock().await;
        tokio_async::R4DCB08::set_address(&mut ctx, new_address).await?;
        ctx.set_slave(Slave(*new_address));
        Ok(())
    }
}
