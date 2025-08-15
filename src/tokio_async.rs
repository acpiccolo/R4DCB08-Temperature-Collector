//! Asynchronous `tokio-modbus` client for the R4DCB08 temperature module.
//!
//! This module provides a high-level API (`R4DCB08` struct) to interact with
//! the R4DCB08 8-channel temperature module using Modbus RTU or TCP. It handles
//! the conversion between Rust types defined in the `crate::protocol` module and
//! the raw Modbus register values.
//!
//! All client methods are `async` and must be `.await`ed.
//!
//! # Examples
//!
//! ## TCP Client Example
//!
//! ```no_run
//! use r4dcb08_lib::tokio_async::R4DCB08;
//! use std::net::SocketAddr;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let socket_addr: SocketAddr = "127.0.0.1:502".parse()?;
//!
//!     // Connect to the Modbus TCP device
//!     let mut modbus_ctx = tokio_modbus::client::tcp::connect(socket_addr).await?;
//!
//!     // Read temperatures from all 8 channels with a timeout
//!     let result = tokio::time::timeout(
//!         Duration::from_secs(1),
//!         R4DCB08::read_temperatures(&mut modbus_ctx),
//!     )
//!     .await;
//!
//!     match result {
//!         Ok(Ok(temperatures)) => println!("Temperatures: {}", temperatures),
//!         Ok(Err(e)) => eprintln!("Modbus error: {}", e),
//!         Err(e) => eprintln!("Timeout error: {}", e),
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## RTU Client Example
//!
//! ```no_run
//! use r4dcb08_lib::tokio_async::R4DCB08;
//! use r4dcb08_lib::protocol::{Address, BaudRate};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let builder = r4dcb08_lib::tokio_common::serial_port_builder(
//!         "/dev/ttyUSB0", // Or "COM3" on Windows, etc.
//!         &BaudRate::B9600,
//!     );
//!     let port = tokio_serial::SerialStream::open(&builder)?;
//!     let slave = tokio_modbus::Slave(1);
//!     let mut modbus_ctx = tokio_modbus::client::rtu::attach_slave(port, slave);
//!
//!     // Read the device's configured baud rate with a timeout
//!     let result = tokio::time::timeout(
//!         Duration::from_secs(1),
//!         R4DCB08::read_baud_rate(&mut modbus_ctx),
//!     )
//!     .await;
//!
//!     match result {
//!         Ok(Ok(remote_baud_rate)) => println!("Device baud rate: {}", remote_baud_rate),
//!         Ok(Err(e)) => eprintln!("Modbus error: {}", e),
//!         Err(e) => eprintln!("Timeout error: {}", e),
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::{protocol as proto, tokio_common::Result};
use tokio_modbus::prelude::{Reader, Writer};

/// Asynchronous client for interacting with the R4DCB08 temperature module over Modbus.
///
/// This struct provides methods to read sensor data and configure the module's
/// operational parameters by wrapping `tokio-modbus` asynchronous operations.
///
/// All methods that interact with the Modbus device are `async` and return `Future`s.
#[derive(Debug)]
pub struct R4DCB08;

impl R4DCB08 {
    /// Helper function to map tokio result to our result.
    fn map_tokio_result<T>(result: tokio_modbus::Result<T>) -> Result<T> {
        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(err)) => Err(err.into()), // Modbus exception
            Err(err) => Err(err.into()),     // IO error
        }
    }

    /// Helper function to read holding registers and decode them into a specific type.
    async fn read_and_decode<T, F>(
        ctx: &mut tokio_modbus::client::Context,
        address: u16,
        quantity: u16,
        decoder: F,
    ) -> Result<T>
    where
        F: FnOnce(&[u16]) -> std::result::Result<T, proto::Error>,
    {
        Ok(decoder(&Self::map_tokio_result(
            ctx.read_holding_registers(address, quantity).await,
        )?)?)
    }

    /// Reads the current temperatures from all 8 available channels in degrees Celsius (°C).
    ///
    /// If a channel's sensor is not connected or reports an error, the corresponding
    /// `proto::Temperature` value will be `proto::Temperature::NAN`.
    ///
    /// # Returns
    ///
    /// A `Result<proto::Temperatures>` containing the temperatures for all channels,
    /// or a Modbus error.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` if a Modbus communication error occurs (e.g., IO error, timeout handled by wrapper, Modbus exception).
    /// * `tokio_modbus::Error::Transport` with `std::io::ErrorKind::InvalidData` if the device returns
    ///   an unexpected number of registers.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async::R4DCB08;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// let temperatures = tokio::time::timeout(Duration::from_secs(2), R4DCB08::read_temperatures(&mut modbus_ctx)).await??;
    /// println!("Temperatures read successfully:");
    /// for (i, temp) in temperatures.iter().enumerate() {
    ///     println!("  Channel {}: {}", i, temp); // `temp` uses Display impl from protocol
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_temperatures(
        ctx: &mut tokio_modbus::client::Context,
    ) -> Result<proto::Temperatures> {
        Self::read_and_decode(
            ctx,
            proto::Temperatures::ADDRESS,
            proto::Temperatures::QUANTITY,
            proto::Temperatures::decode_from_holding_registers,
        )
        .await
    }

    /// Reads the configured temperature correction values (°C) for all 8 channels.
    ///
    /// A `proto::Temperature` value of `0.0` typically means no correction is applied,
    /// while `proto::Temperature::NAN` might indicate an uninitialized or error state for a correction value if read.
    ///
    /// # Returns
    ///
    /// A `Result<proto::TemperatureCorrection>` containing correction values for each channel,
    /// or a Modbus error.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    /// * `tokio_modbus::Error::Transport` with `std::io::ErrorKind::InvalidData` if the device returns
    ///   an unexpected number of registers.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async::R4DCB08;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// let corrections = tokio::time::timeout(Duration::from_secs(2), R4DCB08::read_temperature_correction(&mut modbus_ctx)).await??;
    /// println!("Temperature correction values: {}", corrections);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_temperature_correction(
        ctx: &mut tokio_modbus::client::Context,
    ) -> Result<proto::TemperatureCorrection> {
        Self::read_and_decode(
            ctx,
            proto::TemperatureCorrection::ADDRESS,
            proto::TemperatureCorrection::QUANTITY,
            proto::TemperatureCorrection::decode_from_holding_registers,
        )
        .await
    }

    /// Sets a temperature correction value for a specific channel.
    ///
    /// The `correction` value will be added to the raw temperature reading by the module.
    /// Setting a correction value of `0.0` effectively disables it for that channel.
    ///
    /// # Arguments
    ///
    /// * `channel` - The `proto::Channel` to configure.
    /// * `correction` - The `proto::Temperature` correction value to apply (in °C).
    ///   This type ensures the temperature value is within the representable range.
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    /// * `tokio_modbus::Error::Transport` with `std::io::ErrorKind::InvalidInput` if the
    ///   `correction` value is `NAN`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async::R4DCB08;
    /// use r4dcb08_lib::protocol::{Channel, Temperature, Error};
    /// use std::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// // Set the temperature correction for channel 3 to +1.3°C.
    /// let channel = Channel::try_from(3)?;
    /// let correction_value = Temperature::try_from(1.3)?;
    ///
    /// tokio::time::timeout(Duration::from_secs(2), R4DCB08::set_temperature_correction(&mut modbus_ctx, channel, correction_value)).await??;
    /// println!("Correction for channel {} set to {}.", channel, correction_value);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_temperature_correction(
        ctx: &mut tokio_modbus::client::Context,
        channel: proto::Channel,
        correction: proto::Temperature,
    ) -> Result<()> {
        Self::map_tokio_result(
            ctx.write_single_register(
                proto::TemperatureCorrection::channel_address(channel),
                proto::TemperatureCorrection::encode_for_write_register(correction)?,
            )
            .await,
        )
    }

    /// Reads the automatic temperature reporting interval.
    ///
    /// An interval of `0` seconds ([`proto::AutomaticReport::DISABLED`]) means automatic reporting is off.
    ///
    /// # Returns
    ///
    /// A ` Result<proto::AutomaticReport>` indicating the configured reporting interval,
    /// or a Modbus error.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    /// * `tokio_modbus::Error::Transport` with `std::io::ErrorKind::InvalidData` if the device returns
    ///   malformed data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async::R4DCB08;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// let report = tokio::time::timeout(Duration::from_secs(2), R4DCB08::read_automatic_report(&mut modbus_ctx)).await??;
    /// if report.is_disabled() {
    ///     println!("Automatic reporting is disabled.");
    /// } else {
    ///     println!("Automatic report interval: {} seconds.", report.as_secs());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_automatic_report(
        ctx: &mut tokio_modbus::client::Context,
    ) -> Result<proto::AutomaticReport> {
        Self::read_and_decode(
            ctx,
            proto::AutomaticReport::ADDRESS,
            proto::AutomaticReport::QUANTITY,
            proto::AutomaticReport::decode_from_holding_registers,
        )
        .await
    }

    /// Sets the automatic temperature reporting interval.
    ///
    /// When enabled (interval > 0), the module may periodically send temperature data
    /// unsolicitedly over the RS485 bus (behavior depends on module firmware).
    ///
    /// # Arguments
    ///
    /// * `report` - The `proto::AutomaticReport` interval (0 = disabled, 1-255 seconds).
    ///   The `proto::AutomaticReport` type ensures the value is within the valid hardware range.
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async::R4DCB08;
    /// use r4dcb08_lib::protocol::{AutomaticReport, Error};
    /// use std::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// let report_interval = AutomaticReport::try_from(Duration::from_secs(10))?;
    ///
    /// tokio::time::timeout(Duration::from_secs(2), R4DCB08::set_automatic_report(&mut modbus_ctx, report_interval)).await??;
    /// println!("Automatic report interval set to 10 seconds.");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_automatic_report(
        ctx: &mut tokio_modbus::client::Context,
        report: proto::AutomaticReport,
    ) -> Result<()> {
        Self::map_tokio_result(
            ctx.write_single_register(
                proto::AutomaticReport::ADDRESS,
                report.encode_for_write_register(),
            )
            .await,
        )
    }

    /// Reads the current Modbus communication baud rate setting from the device.
    ///
    /// # Returns
    ///
    /// A `Result<proto::BaudRate>` containing the configured baud rate,
    /// or a Modbus error.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    /// * `tokio_modbus::Error::Transport` with `std::io::ErrorKind::InvalidData` if the device returns
    ///   an invalid baud rate code.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async::R4DCB08;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// let baud_rate = tokio::time::timeout(Duration::from_secs(2), R4DCB08::read_baud_rate(&mut modbus_ctx)).await??;
    /// println!("Current baud rate: {}", baud_rate);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_baud_rate(
        ctx: &mut tokio_modbus::client::Context,
    ) -> Result<proto::BaudRate> {
        Self::read_and_decode(
            ctx,
            proto::BaudRate::ADDRESS,
            proto::BaudRate::QUANTITY,
            proto::BaudRate::decode_from_holding_registers,
        )
        .await
    }

    /// Sets the Modbus communication baud rate for the device.
    ///
    /// **Important:** The new baud rate setting will only take effect after the
    /// R4DCB08 module is **power cycled** (turned off and then on again).
    ///
    /// # Arguments
    ///
    /// * `baud_rate` - The desired `proto::BaudRate` to set.
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async::R4DCB08;
    /// use r4dcb08_lib::protocol::{BaudRate, Error};
    /// use std::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// // Set the baud rate to 19200.
    /// let new_baud_rate = BaudRate::B19200; // Direct enum variant
    /// // Or from u16:
    /// // let new_baud_rate = BaudRate::try_from(19200)?;
    ///
    /// tokio::time::timeout(Duration::from_secs(2), R4DCB08::set_baud_rate(&mut modbus_ctx, new_baud_rate)).await??;
    /// println!("Baud rate set to {}. Power cycle the device for changes to take effect.", new_baud_rate);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_baud_rate(
        ctx: &mut tokio_modbus::client::Context,
        baud_rate: proto::BaudRate,
    ) -> Result<()> {
        Self::map_tokio_result(
            ctx.write_single_register(
                proto::BaudRate::ADDRESS,
                baud_rate.encode_for_write_register(),
            )
            .await,
        )
    }

    /// Resets the R4DCB08 module to its factory default settings.
    ///
    /// This resets all configurable parameters like Modbus Address, Baud Rate,
    /// Temperature Corrections, etc., to their original defaults.
    ///
    /// **Important:**
    /// * After this command is successfully sent, the module may become unresponsive
    ///   on the Modbus bus until it is power cycled.
    /// * A **power cycle** (turning the device off and then on again) is **required**
    ///   to complete the factory reset process and for the default settings to be applied.
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating if the reset command was sent successfully.
    /// It does not confirm the reset is complete, only that the Modbus write was acknowledged.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors. A timeout error after this
    ///   command might be expected as the device resets and may not send a response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async::R4DCB08;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// println!("Attempting to send factory reset command...");
    /// match tokio::time::timeout(Duration::from_secs(2), R4DCB08::factory_reset(&mut modbus_ctx)).await {
    ///     Ok(Ok(())) => println!("Factory reset command sent. Power cycle the device to complete."),
    ///     Ok(Err(e)) => eprintln!("Modbus error during factory reset: {}", e),
    ///     Err(e) => {
    ///         // After the a successful factory reset we get no response :-(
    ///         println!("Factory reset command sent. Device timed out as expected. Power cycle to complete.");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn factory_reset(ctx: &mut tokio_modbus::client::Context) -> Result<()> {
        Self::map_tokio_result(
            ctx.write_single_register(
                proto::FactoryReset::ADDRESS,
                proto::FactoryReset::encode_for_write_register(),
            )
            .await,
        )
    }

    /// Reads the current Modbus device address (Slave ID) from the module.
    ///
    /// **Important Usage Notes:**
    /// * This command is typically used when the device's
    ///   current address is unknown. To do this, the Modbus request **must be sent to
    ///   the broadcast address ([`proto::Address::BROADCAST`])**.
    /// * **Single Device Only:** Only **one** R4DCB08 module should be connected to the
    ///   Modbus bus when executing this command with the broadcast address. If multiple
    ///   devices are present, they might all respond, leading to data collisions and errors.
    ///
    /// # Returns
    ///
    /// A `Result<proto::Address>` containing the device's configured Modbus address,
    /// or a Modbus error.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    /// * `tokio_modbus::Error::Transport` with `std::io::ErrorKind::InvalidData` if the device returns
    ///   a malformed or out-of-range address.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use r4dcb08_lib::tokio_async::R4DCB08;
    /// use r4dcb08_lib::protocol;
    /// use std::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Requires serial port features enabled in tokio-modbus
    /// let builder = tokio_serial::new("/dev/ttyUSB0", 9600) // Baud rate 9600
    ///    .parity(tokio_serial::Parity::None)
    ///    .stop_bits(tokio_serial::StopBits::One)
    ///    .data_bits(tokio_serial::DataBits::Eight)
    ///    .flow_control(tokio_serial::FlowControl::None);
    ///
    /// let port = tokio_serial::SerialStream::open(&builder)?;
    /// // Assume only one device connected, use broadcast address for reading
    /// let mut modbus_ctx = tokio_modbus::client::rtu::attach_slave(port, tokio_modbus::Slave(*protocol::Address::BROADCAST));
    ///
    /// println!("Attempting to read device address using broadcast...");
    /// let device_address = tokio::time::timeout(Duration::from_secs(2), R4DCB08::read_address(&mut modbus_ctx)).await??;
    /// println!("Successfully read device address: {}", device_address);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_address(ctx: &mut tokio_modbus::client::Context) -> Result<proto::Address> {
        Self::read_and_decode(
            ctx,
            proto::Address::ADDRESS,
            proto::Address::QUANTITY,
            proto::Address::decode_from_holding_registers,
        )
        .await
    }

    /// Sets a new Modbus device address.
    ///
    /// **Warning:**
    /// * This permanently changes the device's Modbus address.
    /// * This command must be sent while addressing the device using its **current** Modbus address.
    /// * After successfully changing the address, subsequent communication with the
    ///   device **must** use the new address.
    ///
    /// # Arguments
    ///
    /// * `new_address` - The new `proto::Address` to assign to the device.
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use r4dcb08_lib::tokio_async::R4DCB08;
    /// use r4dcb08_lib::protocol::{Address, Error};
    /// use std::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Requires serial port features enabled in tokio-modbus
    /// let builder = tokio_serial::new("/dev/ttyUSB0", 9600) // Baud rate 9600
    ///    .parity(tokio_serial::Parity::None)
    ///    .stop_bits(tokio_serial::StopBits::One)
    ///    .data_bits(tokio_serial::DataBits::Eight)
    ///    .flow_control(tokio_serial::FlowControl::None);
    ///
    /// // --- Assume device is currently at address 1 ---
    /// let current_device_address = Address::try_from(1)?;
    ///
    /// let port = tokio_serial::SerialStream::open(&builder)?;
    /// let mut modbus_ctx = tokio_modbus::client::rtu::attach_slave(port, tokio_modbus::Slave(*current_device_address));
    ///
    /// // --- New address we want to set ---
    /// let new_device_address = Address::try_from(10)?;
    ///
    /// println!("Attempting to change device address from {} to {}...", current_device_address, new_device_address);
    /// tokio::time::timeout(Duration::from_secs(2), R4DCB08::set_address(&mut modbus_ctx, new_device_address)).await??;
    /// println!("Device address successfully changed to {}.", new_device_address);
    /// println!("You will need to reconnect using the new address for further communication.");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_address(
        ctx: &mut tokio_modbus::client::Context,
        new_address: proto::Address,
    ) -> Result<()> {
        Self::map_tokio_result(
            ctx.write_single_register(
                proto::Address::ADDRESS,
                new_address.encode_for_write_register(),
            )
            .await,
        )
    }
}
