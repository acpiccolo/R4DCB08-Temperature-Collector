use crate::{imp, Error};
use std::time::Duration;
use tokio_modbus::prelude::{Reader, Writer};

type Result<T> = std::result::Result<T, Error>;

pub struct R4DCB08 {
    ctx: tokio_modbus::client::Context,
}

impl R4DCB08 {
    /// Constructs a new R4DCB08 client
    pub fn new(ctx: tokio_modbus::client::Context) -> Self {
        Self { ctx }
    }

    /// Read the current temperature from all channels in °C.
    /// If a channel is not connected or an error is occurred, NaN is returned.
    ///
    /// The returned temperature is corrected by the temperature correction
    pub async fn read_temperature(&mut self) -> Result<Vec<f32>> {
        let rsp = self
            .ctx
            .read_holding_registers(0x0000, imp::NUMBER_OF_CHANNELS as u16)
            .await?;
        Ok(rsp
            .iter()
            .map(|value| imp::degree_celsius_decode(*value))
            .collect::<Vec<_>>())
    }

    /// Read the current temperature correction values form all channels in °C.
    pub async fn read_temperature_correction(&mut self) -> Result<Vec<f32>> {
        let rsp = self
            .ctx
            .read_holding_registers(0x0008, imp::NUMBER_OF_CHANNELS as u16)
            .await?;
        Ok(rsp
            .iter()
            .map(|value| imp::degree_celsius_decode(*value))
            .collect::<Vec<_>>())
    }

    /// Set the temperature correction value per channel.
    ///
    /// The temperature sensor may have an error with the actual temperature.
    /// This correction value can correct the error. The unit is 0.1 °C.
    /// If the correction value is a positive number, the value is added at the current temperature,
    /// and if it is a negative number, the value is subtracted.
    /// Setting it to 0.0 disables this feature.
    ///
    /// * 'channel' - Temperature sensore channel 0 to 7.
    /// * 'correction' - Correction value in °Celsius
    pub async fn set_temperature_correction(&mut self, channel: u8, correction: f32) -> Result<()> {
        imp::write_temperature_correction_check_channel(channel)?;
        Ok(self
            .ctx
            .write_single_register(
                0x0008 + channel as u16,
                imp::degree_celsius_encode(correction)?,
            )
            .await?)
    }

    /// Read temperature automatic reporting
    pub async fn read_automatic_report(&mut self) -> Result<Duration> {
        let rsp = self.ctx.read_holding_registers(0x00FD, 2).await?;
        Ok(imp::read_automatic_report_decode_duration(
            *rsp.first().expect("Result on success expected"),
        ))
    }

    /// Set temperature automatic reporting
    ///
    /// The value is set for all 8 channels at the same time.
    ///
    /// * 'report_in_sec' - Report time in seconds. 0 = disabled (default) or from 1 to 255 seconds.
    pub async fn set_automatic_report(&mut self, report: Duration) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(0x00FD, imp::write_automatic_report_encode_duration(report)?)
            .await?)
    }

    /// Read the current baud rate
    pub async fn read_baud_rate(&mut self) -> Result<imp::BaudRate> {
        let rsp = self.ctx.read_holding_registers(0x00FF, 2).await?;
        Ok(imp::BaudRate::decode(
            *rsp.first().expect("Result on success expected"),
        ))
    }

    /// Set the baud rate.
    ///
    /// Note: The baud rate will be updated when the module is powered up again!
    pub async fn set_baud_rate(&mut self, baud_rate: imp::BaudRate) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(0x00FF, baud_rate.encode())
            .await?)
    }

    /// Reset the device to the factory default settings.
    pub async fn factory_reset(&mut self) -> Result<()> {
        Ok(self.ctx.write_single_register(0x00FF, 5).await?)
    }

    /// Reads the current Modbus address
    ///
    /// Note: When using this command, only one temperature module can be connected to the RS485 bus,
    /// more than one will be wrong!
    /// The connected modbus address must be the broadcast address 255.
    pub async fn read_address(&mut self) -> Result<u8> {
        let rsp = self.ctx.read_holding_registers(0x00FE, 1).await?;
        Ok(*rsp.first().expect("Result on success expected") as u8)
    }

    /// Set the Modbus address
    ///
    /// * 'address' - The address can be from 1 to 247.
    pub async fn set_address(&mut self, address: u8) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(0x00FE, imp::write_address_encode_address(address)?)
            .await?)
    }
}
