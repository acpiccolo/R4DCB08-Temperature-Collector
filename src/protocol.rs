use crate::Error;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BaudRate {
    B1200 = 0,
    B2400 = 1,
    B4800 = 2,
    B9600 = 3,
    B19200 = 4,
}

impl BaudRate {
    pub fn decode(value: u16) -> Self {
        match value {
            0 => BaudRate::B1200,
            1 => BaudRate::B2400,
            2 => BaudRate::B4800,
            3 => BaudRate::B9600,
            4 => BaudRate::B19200,
            5 => unreachable!("Factory reset"),
            _ => unreachable!(),
        }
    }

    pub fn encode(&self) -> u16 {
        *self as u16
    }
}

pub const NUMBER_OF_CHANNELS: u8 = 8;
pub const FACTORY_DEFAULT_BAUD_RATE: &BaudRate = &BaudRate::B9600;
pub const FACTORY_DEFAULT_ADDRESS: u8 = 0x01;
pub const READ_ADDRESS_BROADCAST_ADDRESS: u8 = 0xFF;

pub const READ_TEMPERATURE_REG_ADDR: u16 = 0x0000;
pub const READ_TEMPERATURE_REG_QUAN: u16 = NUMBER_OF_CHANNELS as u16;

pub const READ_TEMPERATURE_CORRECTION_REG_ADDR: u16 = 0x0008;
pub const READ_TEMPERATURE_CORRECTION_REG_QUAN: u16 = NUMBER_OF_CHANNELS as u16;
pub const WRITE_TEMPERATURE_CORRECTION_REG_ADDR: u16 = 0x0008;

pub const READ_AUTOMATIC_REPORT_REG_ADDR: u16 = 0x00FD;
pub const READ_AUTOMATIC_REPORT_REG_QUAN: u16 = 2;
pub const WRITE_AUTOMATIC_REPORT_REG_ADDR: u16 = 0x00FD;

pub const READ_BAUD_RATE_REG_ADDR: u16 = 0x00FF;
pub const READ_BAUD_RATE_REG_QUAN: u16 = 2;
pub const WRITE_BAUD_RATE_REG_ADDR: u16 = 0x00FF;

pub const WRITE_FACTORY_RESET_REG_ADDR: u16 = 0x00FF;
pub const WRITE_FACTORY_RESET_REG_DATA: u16 = 5;

pub const READ_ADDRESS_REG_ADDR: u16 = 0x00FE;
pub const READ_ADDRESS_REG_QUAN: u16 = 1;
pub const WRITE_ADDRESS_REG_ADDR: u16 = 0x00FE;

pub fn degree_celsius_decode(value: u16) -> f32 {
    match value.cmp(&0x8000) {
        std::cmp::Ordering::Greater => {
            // The highest bit 1 indicates a negative value，
            // this value directly subtracting 65536 and divided by 10,
            // is the current temperature value.
            ((value as f32) - 65536.0) / 10.0
        }
        std::cmp::Ordering::Less => {
            // The highest bit is 0, so the temperature is positive,
            // it is converted to decimal and divided by 10
            value as f32 / 10.0
        }
        std::cmp::Ordering::Equal => {
            // When the data is 0X8000(32768), it indicates no sensor or error
            f32::NAN
        }
    }
}

pub const DEGREE_CELSIUS_MIN: f32 = -3276.7;
pub const DEGREE_CELSIUS_MAX: f32 = 3276.7;
pub fn degree_celsius_encode(value: f32) -> std::result::Result<u16, Error> {
    if !(DEGREE_CELSIUS_MIN..=DEGREE_CELSIUS_MAX).contains(&value) {
        Err(Error::DegreeCelciusOutOfRange(value))
    } else if value >= 0.0 {
        Ok((value * 10.0) as u16)
    } else {
        Ok((65536.0 + value * 10.0) as u16)
    }
}

pub const CHANNELS_MIN: u8 = 0;
pub const CHANNELS_MAX: u8 = NUMBER_OF_CHANNELS - 1;
pub fn write_temperature_correction_check_channel(channel: u8) -> std::result::Result<(), Error> {
    if (CHANNELS_MIN..=CHANNELS_MAX).contains(&channel) {
        Ok(())
    } else {
        Err(Error::ChannelOutOfRange(channel))
    }
}

pub const DURATION_MIN: u8 = 0;
pub const DURATION_MAX: u8 = 255;
pub fn read_automatic_report_decode_duration(value: u16) -> Duration {
    Duration::from_secs(value as u64)
}
pub fn write_automatic_report_encode_duration(value: Duration) -> Result<u16, Error> {
    if (DURATION_MIN as u64..=DURATION_MAX as u64).contains(&value.as_secs()) {
        Ok(value.as_secs().try_into().unwrap())
    } else {
        Err(Error::DurationOutOfRange(value.as_secs() as u8))
    }
}

pub const ADDRESS_MIN: u8 = 1;
pub const ADDRESS_MAX: u8 = 247;
pub fn write_address_encode_address(address: u8) -> std::result::Result<u16, Error> {
    if (ADDRESS_MIN..=ADDRESS_MAX).contains(&address) {
        Ok(address as u16)
    } else {
        Err(Error::AddressOutOfRange(address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degree_celsius() {
        assert_eq!(degree_celsius_decode(219), 21.9);
        assert!(matches!(degree_celsius_encode(21.9), Ok(219)));

        assert_eq!(degree_celsius_decode(65424), -11.2);
        assert!(matches!(degree_celsius_encode(-11.2), Ok(65424)));

        assert_eq!(degree_celsius_decode(100), 10.0);
        assert!(matches!(degree_celsius_encode(10.0), Ok(100)));

        assert_eq!(degree_celsius_decode(65506), -3.0);
        assert!(matches!(degree_celsius_encode(-3.0), Ok(65506)));

        // maximum
        assert_eq!(degree_celsius_decode(32767), 3276.7);
        assert!(matches!(degree_celsius_encode(3276.7), Ok(32767)));
        assert!(matches!(
            degree_celsius_encode(3276.8),
            Err(Error::DegreeCelciusOutOfRange(..))
        ));

        // minimum
        assert_eq!(degree_celsius_decode(32769), -3276.7);
        assert!(matches!(degree_celsius_encode(-3276.7), Ok(32769)));
        assert!(matches!(
            degree_celsius_encode(-3276.8),
            Err(Error::DegreeCelciusOutOfRange(..))
        ));

        assert!(degree_celsius_decode(32768).is_nan());
    }

    #[test]
    fn write_address_encode_address_test() {
        assert!(matches!(
            write_address_encode_address(0),
            Err(Error::AddressOutOfRange(..))
        ));
        assert!(matches!(write_address_encode_address(1), Ok(1)));
        assert!(matches!(write_address_encode_address(247), Ok(247)));
        assert!(matches!(
            write_address_encode_address(248),
            Err(Error::AddressOutOfRange(..))
        ));
    }

    #[test]
    fn write_temperature_correction_check_channel_test() {
        assert!(matches!(
            write_temperature_correction_check_channel(0),
            Ok(())
        ));
        assert!(matches!(
            write_temperature_correction_check_channel(7),
            Ok(())
        ));
        assert!(matches!(
            write_temperature_correction_check_channel(8),
            Err(Error::ChannelOutOfRange(..))
        ));
    }

    #[test]
    fn write_automatic_report_encode_duration_test() {
        assert_eq!(read_automatic_report_decode_duration(0), Duration::ZERO);
        assert_eq!(
            read_automatic_report_decode_duration(1),
            Duration::from_secs(1)
        );
        assert_eq!(
            read_automatic_report_decode_duration(255),
            Duration::from_secs(255)
        );

        assert!(matches!(
            write_automatic_report_encode_duration(Duration::ZERO),
            Ok(0)
        ));
        assert!(matches!(
            write_automatic_report_encode_duration(Duration::from_millis(1234)),
            Ok(1)
        ));
        assert!(matches!(
            write_automatic_report_encode_duration(Duration::from_millis(1999)),
            Ok(1)
        ));
        assert!(matches!(
            write_automatic_report_encode_duration(Duration::from_secs(255)),
            Ok(255)
        ));
        assert!(matches!(
            write_automatic_report_encode_duration(Duration::from_secs(256)),
            Err(Error::DurationOutOfRange(..))
        ));
    }
}
