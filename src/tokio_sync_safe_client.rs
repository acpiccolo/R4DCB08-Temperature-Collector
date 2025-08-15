//! Synchronous `tokio-modbus` client for the R4DCB08 temperature module.
//!
//! This module provides a high-level API (`SafeClient` struct) to interact with
//! the R4DCB08 8-channel temperature module using Modbus RTU or TCP. It handles
//! the conversion between Rust types defined in the `crate::protocol` module and
//! the raw Modbus register values.

use crate::{protocol as proto, tokio_common::Result, tokio_sync};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio_modbus::client::sync::Context;

/// Synchronous client for interacting with the R4DCB08 temperature module over Modbus.
///
/// This struct provides methods to read sensor data and configure the module's
/// operational parameters by wrapping `tokio-modbus` synchronous operations.
#[derive(Debug, Clone)]
pub struct SafeClient {
    ctx: Arc<Mutex<Context>>,
}

impl SafeClient {
    /// Creates a new `SafeClient` with a given `tokio-modbus` synchronous context.
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx: Arc::new(Mutex::new(ctx)),
        }
    }

    /// Creates a new `SafeClient` from a shared `tokio-modbus` synchronous context.
    pub fn from_shared(ctx: Arc<Mutex<Context>>) -> Self {
        Self { ctx }
    }

    /// Clones the shared `tokio-modbus` synchronous context.
    pub fn clone_shared(&self) -> Arc<Mutex<Context>> {
        self.ctx.clone()
    }

    /// Sets the timeout for subsequent Modbus read/write operations.
    pub fn set_timeout(&mut self, timeout: impl Into<Option<Duration>>) {
        self.ctx.lock().unwrap().set_timeout(timeout.into());
    }

    /// Retrieves the currently configured timeout for Modbus operations.
    pub fn timeout(&self) -> Option<Duration> {
        self.ctx.lock().unwrap().timeout()
    }

    /// Reads the current temperatures from all 8 available channels in degrees Celsius (°C).
    pub fn read_temperatures(&mut self) -> Result<proto::Temperatures> {
        let mut ctx = self.ctx.lock().unwrap();
        tokio_sync::R4DCB08::read_temperatures(&mut ctx)
    }

    /// Reads the configured temperature correction values (°C) for all 8 channels.
    pub fn read_temperature_correction(&mut self) -> Result<proto::TemperatureCorrection> {
        let mut ctx = self.ctx.lock().unwrap();
        tokio_sync::R4DCB08::read_temperature_correction(&mut ctx)
    }

    /// Sets a temperature correction value for a specific channel.
    pub fn set_temperature_correction(
        &mut self,
        channel: proto::Channel,
        correction: proto::Temperature,
    ) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        tokio_sync::R4DCB08::set_temperature_correction(&mut ctx, channel, correction)
    }

    /// Reads the automatic temperature reporting interval.
    pub fn read_automatic_report(&mut self) -> Result<proto::AutomaticReport> {
        let mut ctx = self.ctx.lock().unwrap();
        tokio_sync::R4DCB08::read_automatic_report(&mut ctx)
    }

    /// Sets the automatic temperature reporting interval.
    pub fn set_automatic_report(&mut self, report: proto::AutomaticReport) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        tokio_sync::R4DCB08::set_automatic_report(&mut ctx, report)
    }

    /// Reads the current Modbus communication baud rate setting from the device.
    pub fn read_baud_rate(&mut self) -> Result<proto::BaudRate> {
        let mut ctx = self.ctx.lock().unwrap();
        tokio_sync::R4DCB08::read_baud_rate(&mut ctx)
    }

    /// Sets the Modbus communication baud rate for the device.
    pub fn set_baud_rate(&mut self, baud_rate: proto::BaudRate) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        tokio_sync::R4DCB08::set_baud_rate(&mut ctx, baud_rate)
    }

    /// Resets the R4DCB08 module to its factory default settings.
    pub fn factory_reset(&mut self) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        tokio_sync::R4DCB08::factory_reset(&mut ctx)
    }

    /// Reads the current Modbus device address (Slave ID) from the module.
    pub fn read_address(&mut self) -> Result<proto::Address> {
        let mut ctx = self.ctx.lock().unwrap();
        tokio_sync::R4DCB08::read_address(&mut ctx)
    }

    /// Sets a new Modbus device address.
    pub fn set_address(&mut self, new_address: proto::Address) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        tokio_sync::R4DCB08::set_address(&mut ctx, new_address)
    }
}
