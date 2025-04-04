use crate::protocol as proto;
use tokio_modbus::prelude::{Reader, Writer};

/// Custom result type for handling errors.
type Result<T> = std::result::Result<T, crate::tokio_error::Error>;

/// Asynchronous client for interacting with the R4DCB08 temperature module over Modbus.
///
/// This struct provides methods to communicate with the module, including reading temperatures,
/// configuring settings, and modifying operational parameters.
pub struct R4DCB08 {
    ctx: tokio_modbus::client::Context,
}

impl R4DCB08 {
    /// Creates a new R4DCB08 client with the given Modbus context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - An asynchronous Modbus client context.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use tokio_modbus::prelude::*;
    /// use r4dcb08_lib::tokio_sync_client::R4DCB08;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    /// let ctx = tokio_modbus::client::sync::tcp::connect("127.0.0.1:502".parse()?)?;
    /// let mut client = R4DCB08::new(ctx);
    /// let temperatures = client.read_temperatures()?;
    /// println!("Temperatures in °C: {}", temperatures);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(ctx: tokio_modbus::client::Context) -> Self {
        Self { ctx }
    }

    /// Reads the current temperatures from all available channels in degrees Celsius (°C).
    ///
    /// If a channel is not connected or an error occurs, NaN is returned for that channel.
    ///
    /// # Returns
    ///
    /// A `Result<proto::Temperatures>` containing the temperatures for all channels.
    pub async fn read_temperatures(&mut self) -> Result<proto::Temperatures> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::Temperatures::ADDRESS, proto::Temperatures::QUANTITY)
            .await??;
        Ok(proto::Temperatures::decode_from_holding_registers(&rsp))
    }

    /// Reads the configured temperature correction values for all channels.
    ///
    /// # Returns
    ///
    /// A `Result<proto::TemperatureCorrection>` containing correction values per channel.
    pub async fn read_temperature_correction(&mut self) -> Result<proto::TemperatureCorrection> {
        let rsp = self
            .ctx
            .read_holding_registers(
                proto::TemperatureCorrection::ADDRESS,
                proto::TemperatureCorrection::QUANTITY,
            )
            .await??;
        Ok(proto::TemperatureCorrection::decode_from_holding_registers(
            &rsp,
        ))
    }

    /// Sets a temperature correction value for a specific channel.
    ///
    /// # Arguments
    ///
    /// * `channel` - The temperature sensor channel.
    /// * `correction` - The correction value in °C. Positive values are added, negative values are subtracted.
    ///   A value of 0.0 disables this correction.
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_sync_client::R4DCB08;
    /// use r4dcb08_lib::protocol::{Channel, Temperature};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    /// # let ctx = tokio_modbus::client::sync::tcp::connect("127.0.0.1:502".parse()?)?;
    /// # let mut client = R4DCB08::new(ctx);
    /// // Set the temperature correction for temperature sensor channel 3 to 1.3°C.
    /// let channel = Channel::try_from(3)?;
    /// let temperature = Temperature::try_from(1.3)?;
    /// client.set_temperature_correction(channel, temperature)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_temperature_correction(
        &mut self,
        channel: proto::Channel,
        correction: proto::Temperature,
    ) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(
                proto::TemperatureCorrection::channel_address(channel),
                proto::TemperatureCorrection::encode_for_write_register(correction),
            )
            .await??)
    }

    /// Reads the automatic temperature reporting interval.
    ///
    /// # Returns
    ///
    /// A `Result<proto::AutomaticReport>` indicating the configured reporting interval.
    pub async fn read_automatic_report(&mut self) -> Result<proto::AutomaticReport> {
        let rsp = self
            .ctx
            .read_holding_registers(
                proto::AutomaticReport::ADDRESS,
                proto::AutomaticReport::QUANTITY,
            )
            .await??;
        Ok(proto::AutomaticReport::decode_from_holding_registers(&rsp))
    }

    /// Sets the automatic temperature reporting interval.
    ///
    /// # Arguments
    ///
    /// * `report` - The reporting interval in seconds (0 = disabled, 1-255 seconds).
    pub async fn set_automatic_report(&mut self, report: proto::AutomaticReport) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(
                proto::AutomaticReport::ADDRESS,
                report.encode_for_write_register(),
            )
            .await??)
    }

    /// Reads the current Modbus baud rate.
    ///
    /// # Returns
    ///
    /// A `Result<proto::BaudRate>` containing the baud rate setting.
    pub async fn read_baud_rate(&mut self) -> Result<proto::BaudRate> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::BaudRate::ADDRESS, proto::BaudRate::QUANTITY)
            .await??;
        Ok(proto::BaudRate::decode_from_holding_registers(&rsp))
    }

    /// Sets the Modbus baud rate.
    ///
    /// **Note:** The new baud rate takes effect after a power cycle.
    ///
    /// # Arguments
    ///
    /// * `baud_rate` - The desired baud rate.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use r4dcb08_lib::tokio_sync_client::R4DCB08;
    /// use r4dcb08_lib::protocol::BaudRate;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    /// # let ctx = tokio_modbus::client::sync::tcp::connect("127.0.0.1:502".parse()?)?;
    /// # let mut client = R4DCB08::new(ctx);
    /// // Set the baud rate to 9600.
    /// let baud_rate = BaudRate::try_from(9600)?;
    /// client.set_baud_rate(baud_rate)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_baud_rate(&mut self, baud_rate: proto::BaudRate) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(
                proto::BaudRate::ADDRESS,
                baud_rate.encode_for_write_register(),
            )
            .await??)
    }

    /// Resets the device to factory default settings.
    ///
    /// **Note:** After this operation, the device will no longer be responsive! To complete the reset, you must turn the power off and on again.
    pub async fn factory_reset(&mut self) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(
                proto::FactoryReset::ADDRESS,
                proto::FactoryReset::encode_for_write_register(),
            )
            .await??)
    }

    /// Reads the current Modbus device address.
    ///
    /// **Warning:** Ensure only one module is connected when using this command.
    /// The connected Modbus address must be set to [proto::Address::BROADCAST].
    ///
    /// # Returns
    ///
    /// A `Result<proto::Address>` containing the Modbus address.
    pub async fn read_address(&mut self) -> Result<proto::Address> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::Address::ADDRESS, proto::Address::QUANTITY)
            .await??;
        Ok(proto::Address::decode_from_holding_registers(&rsp))
    }

    /// Sets a new Modbus device address.
    ///
    /// # Arguments
    ///
    /// * `address` - The new address.
    pub async fn set_address(&mut self, address: proto::Address) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(proto::Address::ADDRESS, address.encode_for_write_register())
            .await??)
    }
}
