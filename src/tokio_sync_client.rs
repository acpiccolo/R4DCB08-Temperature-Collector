use crate::{protocol as proto, Error};
use std::time::Duration;
use tokio_modbus::prelude::{SyncReader, SyncWriter};

type Result<T> = std::result::Result<T, Error>;

pub struct R4DCB08 {
    ctx: tokio_modbus::client::sync::Context,
}

impl R4DCB08 {
    /// Constructs a new R4DCB08 client
    pub fn new(ctx: tokio_modbus::client::sync::Context) -> Self {
        Self { ctx }
    }

    /// Sets the modbus context timeout.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.ctx.set_timeout(timeout);
    }

    pub fn timeout(&self) -> Option<Duration> {
        self.ctx.timeout()
    }

    /// Read the current temperature from all channels in °C.
    /// If a channel is not connected or an error is occurred, NaN is returned.
    ///
    /// The returned temperature is corrected by the temperature correction
    pub fn read_temperature(&mut self) -> Result<Vec<f32>> {
        let rsp = self.ctx.read_holding_registers(
            proto::READ_TEMPERATURE_REG_ADDR,
            proto::READ_TEMPERATURE_REG_QUAN,
        )?;
        Ok(rsp
            .iter()
            .map(|value| proto::degree_celsius_decode(*value))
            .collect::<Vec<_>>())
    }

    /// Read the current temperature correction values form all channels in °C.
    pub fn read_temperature_correction(&mut self) -> Result<Vec<f32>> {
        let rsp = self.ctx.read_holding_registers(
            proto::READ_TEMPERATURE_CORRECTION_REG_ADDR,
            proto::READ_TEMPERATURE_CORRECTION_REG_QUAN,
        )?;
        Ok(rsp
            .iter()
            .map(|value| proto::degree_celsius_decode(*value))
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
    pub fn set_temperature_correction(&mut self, channel: u8, correction: f32) -> Result<()> {
        proto::write_temperature_correction_check_channel(channel)?;
        Ok(self.ctx.write_single_register(
            proto::WRITE_TEMPERATURE_CORRECTION_REG_ADDR + channel as u16,
            proto::degree_celsius_encode(correction)?,
        )?)
    }

    /// Read temperature automatic reporting
    pub fn read_automatic_report(&mut self) -> Result<Duration> {
        let rsp = self.ctx.read_holding_registers(
            proto::READ_AUTOMATIC_REPORT_REG_ADDR,
            proto::READ_AUTOMATIC_REPORT_REG_QUAN,
        )?;
        Ok(proto::read_automatic_report_decode_duration(
            *rsp.first().expect("Result on success expected"),
        ))
    }

    /// Set temperature automatic reporting
    ///
    /// The value is set for all 8 channels at the same time.
    ///
    /// * 'report_in_sec' - Report time in seconds. 0 = disabled (default) or from 1 to 255 seconds.
    pub fn set_automatic_report(&mut self, report: Duration) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::WRITE_AUTOMATIC_REPORT_REG_ADDR,
            proto::write_automatic_report_encode_duration(report)?,
        )?)
    }

    /// Read the current baud rate
    pub fn read_baud_rate(&mut self) -> Result<proto::BaudRate> {
        let rsp = self.ctx.read_holding_registers(
            proto::READ_BAUD_RATE_REG_ADDR,
            proto::READ_BAUD_RATE_REG_QUAN,
        )?;
        Ok(proto::BaudRate::decode(
            *rsp.first().expect("Result on success expected"),
        ))
    }

    /// Set the baud rate.
    ///
    /// Note: The baud rate will be updated when the module is powered up again!
    pub fn set_baud_rate(&mut self, baud_rate: proto::BaudRate) -> Result<()> {
        Ok(self
            .ctx
            .write_single_register(proto::WRITE_BAUD_RATE_REG_ADDR, baud_rate.encode())?)
    }

    /// Reset the device to the factory default settings.
    pub fn factory_reset(&mut self) -> Result<()> {
        Ok(self.ctx.write_single_register(
            proto::WRITE_FACTORY_RESET_REG_ADDR,
            proto::WRITE_FACTORY_RESET_REG_DATA,
        )?)
    }

    /// Reads the current Modbus address
    ///
    /// Note: When using this command, only one temperature module can be connected to the RS485 bus,
    /// more than one will be wrong!
    /// The connected modbus address must be the broadcast address 255.
    pub fn read_address(&mut self) -> Result<u8> {
        let rsp = self
            .ctx
            .read_holding_registers(proto::READ_ADDRESS_REG_ADDR, proto::READ_ADDRESS_REG_QUAN)?;
        Ok(*rsp.first().expect("Result on success expected") as u8)
    }

    /// Set the Modbus address
    ///
    /// * 'address' - The address can be from 1 to 247.
    pub fn set_address(&mut self, address: u8) -> Result<()> {
        self.ctx.write_single_register(
            proto::WRITE_ADDRESS_REG_ADDR,
            proto::write_address_encode_address(address)?,
        )?;
        Ok(())
    }
}
