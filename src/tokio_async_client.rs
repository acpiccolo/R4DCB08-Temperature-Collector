use crate::protocol as proto;
use tokio_modbus::prelude::{Reader, Writer};

type Result<T> = std::result::Result<T, crate::tokio_error::Error>;

pub struct R4DCB08 {
    ctx: tokio_modbus::client::Context,
}

impl R4DCB08 {
    /// Constructs a new R4DCB08 client
    pub fn new(ctx: tokio_modbus::client::Context) -> Self {
        Self { ctx }
    }

    /// Read the current temperatures from all channels in °C.
    /// If a channel is not connected or an error is occurred, NaN is returned.
    ///
    /// The returned temperatures is corrected by the temperature correction
    pub async fn read_temperatures(&mut self) -> Result<proto::Temperatures> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::Temperatures::ADDRESS, proto::Temperatures::QUANTITY)
            .await??;
        Ok(proto::Temperatures::decode_from_holding_registers(&rsp))
    }

    /// Read the current temperature correction values form all channels in °C.
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

    /// Set the temperature correction value per channel.
    ///
    /// The temperature sensor may have an error with the actual temperature.
    /// This correction value can correct the error. The unit is 0.1 °C.
    /// If the correction value is a positive number, the value is added at the current temperature,
    /// and if it is a negative number, the value is subtracted.
    /// Setting it to 0.0 disables this feature.
    ///
    /// * 'channel' - Temperature sensor channel 0 to 7.
    /// * 'correction' - Correction value in °Celsius
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

    /// Read temperature automatic reporting
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

    /// Set temperature automatic reporting
    ///
    /// The value is set for all 8 channels at the same time.
    ///
    /// * 'report_in_sec' - Report time in seconds. 0 = disabled (default) or from 1 to 255 seconds.
    pub async fn set_automatic_report(&mut self, report: proto::AutomaticReport) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(
                proto::AutomaticReport::ADDRESS,
                report.encode_for_write_register(),
            )
            .await??)
    }

    /// Read the current baud rate
    pub async fn read_baud_rate(&mut self) -> Result<proto::BaudRate> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::BaudRate::ADDRESS, proto::BaudRate::QUANTITY)
            .await??;
        Ok(proto::BaudRate::decode_from_holding_registers(&rsp))
    }

    /// Set the baud rate.
    ///
    /// Note: The baud rate will be updated when the module is powered up again!
    pub async fn set_baud_rate(&mut self, baud_rate: proto::BaudRate) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(
                proto::BaudRate::ADDRESS,
                baud_rate.encode_for_write_register(),
            )
            .await??)
    }

    /// Reset the device to the factory default settings.
    pub async fn factory_reset(&mut self) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(
                proto::FactoryReset::ADDRESS,
                proto::FactoryReset::encode_for_write_register(),
            )
            .await??)
    }

    /// Reads the current Modbus address
    ///
    /// Note: When using this command, only one temperature module can be connected to the RS485 bus,
    /// more than one will be wrong!
    /// The connected modbus address must be the broadcast address 255.
    pub async fn read_address(&mut self) -> Result<proto::Address> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::Address::ADDRESS, proto::Address::QUANTITY)
            .await??;
        Ok(proto::Address::decode_from_holding_registers(&rsp))
    }

    /// Set the Modbus address
    ///
    /// * 'address' - The address can be from 1 to 247.
    pub async fn set_address(&mut self, address: proto::Address) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(proto::Address::ADDRESS, address.encode_for_write_register())
            .await??)
    }
}
