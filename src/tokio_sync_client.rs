use crate::protocol as proto;
use std::time::Duration;
use tokio_modbus::prelude::{SyncReader, SyncWriter};

type Result<T> = std::result::Result<T, crate::tokio_error::Error>;

pub struct R4DCB08 {
    ctx: tokio_modbus::client::sync::Context,
}

impl R4DCB08 {
    /// Constructs a new R4DCB08 client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use r4dcb08_lib::tokio_sync_client::R4DCB08;
    /// use std::time::Duration;
    ///
    /// let ctx = tokio_modbus::client::sync::tcp::connect("127.0.0.1:502".parse().unwrap()).unwrap();
    /// let mut client = R4DCB08::new(ctx);
    /// client.set_timeout(Duration::from_secs(5));
    /// ```
    pub fn new(ctx: tokio_modbus::client::sync::Context) -> Self {
        Self { ctx }
    }

    /// Sets the Modbus context timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - A `Duration` representing the timeout period.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.ctx.set_timeout(timeout);
    }

    /// Gets the current Modbus context timeout.
    ///
    /// # Returns
    ///
    /// An `Option<Duration>` representing the current timeout if set.
    pub fn timeout(&self) -> Option<Duration> {
        self.ctx.timeout()
    }

    /// Reads the current temperatures from all channels in °C.
    ///
    /// If a channel is not connected or an error occurs, NaN is returned.
    ///
    /// # Returns
    ///
    /// A `Result<proto::Temperatures>` containing the temperatures of all channels.
    pub fn read_temperatures(&mut self) -> Result<proto::Temperatures> {
        let rsp = self.ctx.read_holding_registers(
            proto::Temperatures::ADDRESS,
            proto::Temperatures::QUANTITY,
        )??;
        Ok(proto::Temperatures::decode_from_holding_registers(&rsp))
    }

    /// Reads the current temperature correction values for all channels in °C.
    ///
    /// # Returns
    ///
    /// A `Result<proto::TemperatureCorrection>` containing the correction values for all channels.
    pub fn read_temperature_correction(&mut self) -> Result<proto::TemperatureCorrection> {
        let rsp = self.ctx.read_holding_registers(
            proto::TemperatureCorrection::ADDRESS,
            proto::TemperatureCorrection::QUANTITY,
        )??;
        Ok(proto::TemperatureCorrection::decode_from_holding_registers(
            &rsp,
        ))
    }

    /// Set the temperature correction value per channel.
    ///
    /// The temperature sensor may have an error with the actual temperature.
    /// This correction value can correct the error. The unit is 0.1 °C.
    /// If the correction value is a positive number, the value is added at the current temperature,
    /// and if it is a negative number, the value is subtracted.
    /// Setting it to 0.0 disables this feature.
    ///
    /// # Arguments
    ///
    /// * `channel` - Temperature sensor channel (0 to 7).
    /// * `correction` - Correction value in °C.
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure.
    pub fn set_temperature_correction(
        &mut self,
        channel: proto::Channel,
        correction: proto::Temperature,
    ) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::TemperatureCorrection::channel_address(channel),
            proto::TemperatureCorrection::encode_for_write_register(correction),
        )??)
    }

    /// Reads the automatic temperature reporting interval.
    ///
    /// # Returns
    ///
    /// A `Result<Duration>` containing the reporting interval.
    pub fn read_automatic_report(&mut self) -> Result<proto::AutomaticReport> {
        let rsp = self.ctx.read_holding_registers(
            proto::AutomaticReport::ADDRESS,
            proto::AutomaticReport::QUANTITY,
        )??;
        Ok(proto::AutomaticReport::decode_from_holding_registers(&rsp))
    }

    /// Sets the automatic temperature reporting interval.
    ///
    /// # Arguments
    ///
    /// * `report` - Report interval in seconds (0 = disabled, 1 to 255 seconds).
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure.
    pub fn set_automatic_report(&mut self, report: proto::AutomaticReport) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::AutomaticReport::ADDRESS,
            report.encode_for_write_register(),
        )??)
    }

    /// Reads the current baud rate.
    ///
    /// # Returns
    ///
    /// A `Result<proto::BaudRate>` containing the current baud rate.
    pub fn read_baud_rate(&mut self) -> Result<proto::BaudRate> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::BaudRate::ADDRESS, proto::BaudRate::QUANTITY)??;
        Ok(proto::BaudRate::decode_from_holding_registers(&rsp))
    }

    /// Sets the baud rate.
    ///
    /// Note: The baud rate will be updated when the module is powered up again!
    ///
    /// # Arguments
    ///
    /// * `baud_rate` - The new baud rate to set.
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure.
    pub fn set_baud_rate(&mut self, baud_rate: proto::BaudRate) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::BaudRate::ADDRESS,
            baud_rate.encode_for_write_register(),
        )??)
    }

    /// Resets the device to the factory default settings.
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure.
    pub fn factory_reset(&mut self) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::FactoryReset::ADDRESS,
            proto::FactoryReset::encode_for_write_register(),
        )??)
    }

    /// Reads the current Modbus address.
    ///
    /// Note: When using this command, only one temperature module can be connected to the RS485 bus,
    /// more than one will cause errors! The connected Modbus address must be the broadcast address (255).
    ///
    /// # Returns
    ///
    /// A `Result<u8>` containing the current Modbus address.
    pub fn read_address(&mut self) -> Result<proto::Address> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::Address::ADDRESS, proto::Address::QUANTITY)??;
        Ok(proto::Address::decode_from_holding_registers(&rsp))
    }

    /// Sets the Modbus address.
    ///
    /// # Arguments
    ///
    /// * `address` - The new address (1 to 247).
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure.
    pub fn set_address(&mut self, address: proto::Address) -> Result<()> {
        self.ctx.write_single_register(
            proto::Address::ADDRESS,
            address.encode_for_write_register(),
        )??;
        Ok(())
    }
}
