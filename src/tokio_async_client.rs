//! Asynchronous `tokio-modbus` client for the R4DCB08 temperature module.
//!
//! This module provides a high-level API (`R4DCB08` struct) to interact with
//! the R4DCB08 8-channel temperature module using Modbus RTU or TCP. It handles
//! the conversion between Rust types defined in the `crate::protocol` module and
//! the raw Modbus register values.
//!
//! All client methods are `async` and must be `.await`ed.

use crate::protocol as proto;
use tokio_modbus::prelude::{Reader, Writer};

/// Asynchronous client for interacting with the R4DCB08 temperature module over Modbus.
///
/// This struct provides methods to read sensor data and configure the module's
/// operational parameters by wrapping `tokio-modbus` asynchronous operations.
///
/// All methods that interact with the Modbus device are `async` and return `Future`s.
#[derive(Debug)]
pub struct R4DCB08 {
    ctx: tokio_modbus::client::Context,
}

impl R4DCB08 {
    /// Creates a new `R4DCB08` client with a given `tokio-modbus` asynchronous context.
    ///
    /// The `tokio_modbus::client::Context` should be appropriately configured for the target Modbus slave device
    /// (e.g., with the correct slave ID for RTU, or target IP and port for TCP).
    ///
    /// # Arguments
    ///
    /// * `ctx` - An asynchronous `tokio_modbus::client::Context`.
    ///
    /// # Examples
    ///
    /// **RTU (Serial) Example:**
    /// ```no_run
    /// use r4dcb08_lib::tokio_async_client::R4DCB08;
    /// use r4dcb08_lib::protocol::{Address, BaudRate};
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let tty_path = "/dev/ttyUSB0"; // Or "COM3" on Windows, etc.
    ///     let slave_id = Address::default(); // Default is 1
    ///
    ///     // Note: Serial port specific timeouts (like inter-character) are configured on the builder.
    ///     // Overall Modbus operation timeouts are typically handled by wrapping calls with `tokio::time::timeout`.
    ///     let builder = tokio_serial::new(tty_path, BaudRate::default().into())
    ///        .parity(tokio_serial::Parity::None)
    ///        .stop_bits(tokio_serial::StopBits::One)
    ///        .data_bits(tokio_serial::DataBits::Eight)
    ///        .flow_control(tokio_serial::FlowControl::None);
    ///
    ///     let port = tokio_serial::SerialStream::open(&builder)?;
    ///     let modbus_ctx = tokio_modbus::client::rtu::attach_slave(port, tokio_modbus::Slave(*slave_id));
    ///     let mut client = R4DCB08::new(modbus_ctx);
    ///
    ///     // Example of using tokio::time::timeout for an operation
    ///     let temps = tokio::time::timeout(Duration::from_secs(2), client.read_temperatures()).await???;
    ///     println!("Temperatures: {}", temps);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// **TCP Example:**
    /// ```no_run
    /// use r4dcb08_lib::tokio_async_client::R4DCB08;
    /// use std::net::SocketAddr;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    ///  async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let socket_addr: SocketAddr = "127.0.0.1:502".parse()?;
    ///     let mut modbus_ctx = tokio_modbus::client::tcp::connect(socket_addr).await?;
    ///     let mut client = R4DCB08::new(modbus_ctx);
    ///     
    ///     let temps = tokio::time::timeout(Duration::from_secs(2), client.read_temperatures()).await???;
    ///     println!("Temperatures: {}", temps);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(ctx: tokio_modbus::client::Context) -> Self {
        Self { ctx }
    }

    // Note: The asynchronous `tokio_modbus::client::Context` does not typically have `set_timeout`
    // or `timeout` methods directly on the context itself. Timeouts for asynchronous operations
    // are generally handled by wrapping the `.await` call with `tokio::time::timeout`,
    // as shown in the examples for other methods.
    // If your specific `tokio_modbus::client::Context` (perhaps a custom wrapper or a specific transport context)
    // does offer these, the documentation would need to be adjusted. For the standard
    // `tokio_modbus::client::Context`, these methods are not present.

    /// Reads the current temperatures from all 8 available channels in degrees Celsius (°C).
    ///
    /// If a channel's sensor is not connected or reports an error, the corresponding
    /// `proto::Temperature` value will be `proto::Temperature::NAN`.
    ///
    /// # Returns
    ///
    /// A `tokio_modbus::Result<proto::Temperatures>` containing the temperatures for all channels,
    /// or a Modbus error.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` if a Modbus communication error occurs (e.g., IO error, timeout handled by wrapper, Modbus exception).
    ///
    /// # Panics
    ///
    /// * This method will panic if `proto::Temperatures::decode_from_holding_registers` panics.
    ///   This can occur if the device returns an incorrect number of registers (if not caught by the Modbus layer)
    ///   or if the data within registers is malformed in a way that the protocol decoder cannot handle
    ///   (though Modbus CRC should prevent most data corruption issues).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async_client::R4DCB08;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// # let mut client = R4DCB08::new(modbus_ctx);
    /// let temperatures = tokio::time::timeout(Duration::from_secs(2), client.read_temperatures()).await???;
    /// println!("Temperatures read successfully:");
    /// for (i, temp) in temperatures.iter().enumerate() {
    ///     println!("  Channel {}: {}", i, temp); // `temp` uses Display impl from protocol
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_temperatures(&mut self) -> tokio_modbus::Result<proto::Temperatures> {
        match self
            .ctx
            .read_holding_registers(proto::Temperatures::ADDRESS, proto::Temperatures::QUANTITY)
            .await
        {
            Ok(Ok(words)) => {
                // Modbus read successful, now decode
                // The protocol's decode_from_holding_registers expects the correct number of words
                // and will panic if the slice length is wrong or if individual word decoding panics.
                Ok(Ok(proto::Temperatures::decode_from_holding_registers(
                    &words,
                )))
            }
            Ok(Err(err)) => {
                // Modbus read returned an error/exception within the Ok variant
                Ok(Err(err))
            }
            Err(err) => {
                // Underlying communication error (e.g., IO error, timeout)
                Err(err)
            }
        }
    }

    /// Reads the configured temperature correction values (°C) for all 8 channels.
    ///
    /// A `proto::Temperature` value of `0.0` typically means no correction is applied,
    /// while `proto::Temperature::NAN` might indicate an uninitialized or error state for a correction value if read.
    ///
    /// # Returns
    ///
    /// A `tokio_modbus::Result<proto::TemperatureCorrection>` containing correction values for each channel,
    /// or a Modbus error.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    ///
    /// # Panics
    ///
    /// * Similar to `read_temperatures`, panics if `proto::TemperatureCorrection::decode_from_holding_registers` panics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async_client::R4DCB08;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// # let mut client = R4DCB08::new(modbus_ctx);
    /// let corrections = tokio::time::timeout(Duration::from_secs(2), client.read_temperature_correction()).await???;
    /// println!("Temperature correction values: {}", corrections);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_temperature_correction(
        &mut self,
    ) -> tokio_modbus::Result<proto::TemperatureCorrection> {
        match self
            .ctx
            .read_holding_registers(
                proto::TemperatureCorrection::ADDRESS,
                proto::TemperatureCorrection::QUANTITY,
            )
            .await
        {
            Ok(Ok(words)) => {
                // Modbus read successful, now decode
                // The protocol's decode_from_holding_registers expects the correct number of words
                // and will panic if the slice length is wrong or if individual word decoding panics.
                Ok(Ok(
                    proto::TemperatureCorrection::decode_from_holding_registers(&words),
                ))
            }
            Ok(Err(err)) => {
                // Modbus read returned an error/exception within the Ok variant
                Ok(Err(err))
            }
            Err(err) => {
                // Underlying communication error (e.g., IO error, timeout)
                Err(err)
            }
        }
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
    ///   **Note:** `proto::Temperature::NAN` cannot be written; the underlying protocol's
    ///   `encode_for_write_register` method will panic.
    ///
    /// # Returns
    ///
    /// A `tokio_modbus::Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    ///
    /// # Panics
    ///
    /// * If `correction` is `proto::Temperature::NAN`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async_client::R4DCB08;
    /// use r4dcb08_lib::protocol::{Channel, Temperature, ErrorDegreeCelsiusOutOfRange, ErrorChannelOutOfRange};
    /// use std::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// # let mut client = R4DCB08::new(modbus_ctx);
    /// // Set the temperature correction for channel 3 to +1.3°C.
    /// let channel = Channel::try_from(3)?; // Or handle ErrorChannelOutOfRange appropriately
    /// let correction_value = Temperature::try_from(1.3)?; // Or handle ErrorDegreeCelsiusOutOfRange
    ///
    /// // It's good practice to check for NAN before attempting to set a correction.
    /// if correction_value.is_nan() {
    ///     eprintln!("Error: Cannot set temperature correction to NAN.");
    ///     return Ok(()); // Or return an appropriate error
    /// }
    ///
    /// tokio::time::timeout(Duration::from_secs(2), client.set_temperature_correction(channel, correction_value)).await???;
    /// println!("Correction for channel {} set to {}.", channel, correction_value);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_temperature_correction(
        &mut self,
        channel: proto::Channel,
        correction: proto::Temperature,
    ) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                proto::TemperatureCorrection::channel_address(channel),
                proto::TemperatureCorrection::encode_for_write_register(correction),
            )
            .await
    }

    /// Reads the automatic temperature reporting interval.
    ///
    /// An interval of `0` seconds ([`proto::AutomaticReport::DISABLED`]) means automatic reporting is off.
    ///
    /// # Returns
    ///
    /// A ` tokio_modbus::Result<proto::AutomaticReport>` indicating the configured reporting interval,
    /// or a Modbus error.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    ///
    /// # Panics
    ///
    /// * If the device response is malformed causing `proto::AutomaticReport::decode_from_holding_registers` to panic.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async_client::R4DCB08;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// # let mut client = R4DCB08::new(modbus_ctx);
    /// let report = tokio::time::timeout(Duration::from_secs(2), client.read_automatic_report()).await???;
    /// if report.is_disabled() {
    ///     println!("Automatic reporting is disabled.");
    /// } else {
    ///     println!("Automatic report interval: {} seconds.", report.as_secs());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_automatic_report(&mut self) -> tokio_modbus::Result<proto::AutomaticReport> {
        match self
            .ctx
            .read_holding_registers(
                proto::AutomaticReport::ADDRESS,
                proto::AutomaticReport::QUANTITY,
            )
            .await
        {
            Ok(Ok(words)) => {
                // Modbus read successful, now decode
                // The protocol's decode_from_holding_registers expects the correct number of words
                // and will panic if the slice length is wrong or if individual word decoding panics.
                Ok(Ok(proto::AutomaticReport::decode_from_holding_registers(
                    &words,
                )))
            }
            Ok(Err(err)) => {
                // Modbus read returned an error/exception within the Ok variant
                Ok(Err(err))
            }
            Err(err) => {
                // Underlying communication error (e.g., IO error, timeout)
                Err(err)
            }
        }
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
    /// A `tokio_modbus::Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async_client::R4DCB08;
    /// use r4dcb08_lib::protocol::{AutomaticReport, ErrorDurationOutOfRange};
    /// use std::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// # let mut client = R4DCB08::new(modbus_ctx);
    /// let report_interval = AutomaticReport::try_from(Duration::from_secs(10))?;
    ///
    /// tokio::time::timeout(Duration::from_secs(2), client.set_automatic_report(report_interval)).await???;
    /// println!("Automatic report interval set to 10 seconds.");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_automatic_report(
        &mut self,
        report: proto::AutomaticReport,
    ) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                proto::AutomaticReport::ADDRESS,
                report.encode_for_write_register(),
            )
            .await
    }

    /// Reads the current Modbus communication baud rate setting from the device.
    ///
    /// # Returns
    ///
    /// A `tokio_modbus::Result<proto::BaudRate>` containing the configured baud rate,
    /// or a Modbus error.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    ///
    /// # Panics
    ///
    /// * If `proto::BaudRate::decode_from_holding_registers` panics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async_client::R4DCB08;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// # let mut client = R4DCB08::new(modbus_ctx);
    /// let baud_rate = tokio::time::timeout(Duration::from_secs(2), client.read_baud_rate()).await???;
    /// println!("Current baud rate: {}", baud_rate);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_baud_rate(&mut self) -> tokio_modbus::Result<proto::BaudRate> {
        match self
            .ctx
            .read_holding_registers(proto::BaudRate::ADDRESS, proto::BaudRate::QUANTITY)
            .await
        {
            Ok(Ok(words)) => {
                // Modbus read successful, now decode
                // The protocol's decode_from_holding_registers expects the correct number of words
                // and will panic if the slice length is wrong or if individual word decoding panics.
                Ok(Ok(proto::BaudRate::decode_from_holding_registers(&words)))
            }
            Ok(Err(err)) => {
                // Modbus read returned an error/exception within the Ok variant
                Ok(Err(err))
            }
            Err(err) => {
                // Underlying communication error (e.g., IO error, timeout)
                Err(err)
            }
        }
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
    /// A `tokio_modbus::Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_async_client::R4DCB08;
    /// use r4dcb08_lib::protocol::{BaudRate, ErrorInvalidBaudRate};
    /// use std::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// # let mut client = R4DCB08::new(modbus_ctx);
    /// // Set the baud rate to 19200.
    /// let new_baud_rate = BaudRate::B19200; // Direct enum variant
    /// // Or from u16:
    /// // let new_baud_rate = BaudRate::try_from(19200)?;
    ///
    /// tokio::time::timeout(Duration::from_secs(2), client.set_baud_rate(new_baud_rate)).await???;
    /// println!("Baud rate set to {}. Power cycle the device for changes to take effect.", new_baud_rate);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_baud_rate(&mut self, baud_rate: proto::BaudRate) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                proto::BaudRate::ADDRESS,
                baud_rate.encode_for_write_register(),
            )
            .await
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
    /// A `tokio_modbus::Result<()>` indicating if the reset command was sent successfully.
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
    /// # use r4dcb08_lib::tokio_async_client::R4DCB08;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let modbus_ctx = tokio_modbus::client::tcp::connect("127.0.0.1:502".parse()?).await?;
    /// # let mut client = R4DCB08::new(modbus_ctx);
    /// println!("Attempting to send factory reset command...");
    /// match tokio::time::timeout(Duration::from_secs(2), client.factory_reset()).await {
    ///     Ok(Ok(Ok(()))) => println!("Factory reset command sent. Power cycle the device to complete."),
    ///     Ok(Ok(Err(e))) => eprintln!("Modbus error during factory reset: {}", e),
    ///     Ok(Err(e)) => eprintln!("Modbus error during factory reset: {}", e),
    ///     Err(e) => {
    ///         // After the a successful factory reset we get no response :-(
    ///         println!("Factory reset command sent. Device timed out as expected. Power cycle to complete.");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn factory_reset(&mut self) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                proto::FactoryReset::ADDRESS,
                proto::FactoryReset::encode_for_write_register(),
            )
            .await
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
    /// A `tokio_modbus::Result<proto::Address>` containing the device's configured Modbus address,
    /// or a Modbus error.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    ///
    /// # Panics
    ///
    /// * If `proto::Address::decode_from_holding_registers` panics.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use r4dcb08_lib::tokio_async_client::R4DCB08;
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
    /// let modbus_ctx = tokio_modbus::client::rtu::attach_slave(port, tokio_modbus::Slave(*protocol::Address::BROADCAST));
    /// let mut client = R4DCB08::new(modbus_ctx);
    ///
    /// println!("Attempting to read device address using broadcast...");
    /// let device_address = tokio::time::timeout(Duration::from_secs(2), client.read_address()).await???;
    /// println!("Successfully read device address: {}", device_address);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_address(&mut self) -> tokio_modbus::Result<proto::Address> {
        // Caller is responsible for ensuring self.ctx targets Address::BROADCAST
        // if the current device address is unknown.
        match self
            .ctx
            .read_holding_registers(proto::Address::ADDRESS, proto::Address::QUANTITY)
            .await
        {
            Ok(Ok(words)) => {
                // Modbus read successful, now decode
                // The protocol's decode_from_holding_registers expects the correct number of words
                // and will panic if the slice length is wrong or if individual word decoding panics
                // or if value is invalid (e.g. 0 or > 247).
                Ok(Ok(proto::Address::decode_from_holding_registers(&words)))
            }
            Ok(Err(err)) => {
                // Modbus read returned an error/exception within the Ok variant
                Ok(Err(err))
            }
            Err(err) => {
                // Underlying communication error (e.g., IO error, timeout)
                Err(err)
            }
        }
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
    /// A `tokio_modbus::Result<()>` indicating success or failure of the write operation.
    ///
    /// # Errors
    ///
    /// * `tokio_modbus::Error` for Modbus communication errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use r4dcb08_lib::tokio_async_client::R4DCB08;
    /// use r4dcb08_lib::protocol::{Address, ErrorAddressOutOfRange};
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
    /// let modbus_ctx = tokio_modbus::client::rtu::attach_slave(port, tokio_modbus::Slave(*current_device_address));
    /// let mut client = R4DCB08::new(modbus_ctx);
    ///
    /// // --- New address we want to set ---
    /// let new_device_address = Address::try_from(10)?;
    ///
    /// println!("Attempting to change device address from {} to {}...", current_device_address, new_device_address);
    /// tokio::time::timeout(Duration::from_secs(2), client.set_address(new_device_address)).await???;
    /// println!("Device address successfully changed to {}.", new_device_address);
    /// println!("You will need to reconnect using the new address for further communication.");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_address(&mut self, new_address: proto::Address) -> tokio_modbus::Result<()> {
        self.ctx
            .write_single_register(
                proto::Address::ADDRESS,
                new_address.encode_for_write_register(),
            )
            .await
    }
}
