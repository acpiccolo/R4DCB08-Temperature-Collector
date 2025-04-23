use std::time::Duration;
use thiserror::Error;

/// Number of channels available in the R4DCB08 temperature module.
pub const NUMBER_OF_CHANNELS: usize = 8;

/// 16-bit value stored in Modbus register.
pub type Word = u16;

/// Enum representing the baud rates for communication.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BaudRate {
    /// 1200 baud rate.
    B1200,
    /// 2400 baud rate.
    B2400,
    /// 4800 baud rate.
    B4800,
    /// Factory default baud rate for the R4DCB08 temperature module.
    #[default]
    B9600,
    /// 19200 baud rate.
    B19200,
}

impl BaudRate {
    /// Register address for reading and writing the baud rate.
    pub const ADDRESS: u16 = 0x00FF;
    /// Number of registers to read for the baud rate.
    pub const QUANTITY: u16 = 1;

    /// Decodes a `BaudRate` from a Modbus holding register value.
    ///
    /// # Arguments
    ///
    /// * `value` - A slice of `Word` containing the register value.
    ///
    /// # Returns
    ///
    /// A `BaudRate` enum containing the decoded value.
    pub fn decode_from_holding_registers(value: &[Word]) -> Self {
        match value.first().unwrap() {
            0 => Self::B1200,
            1 => Self::B2400,
            2 => Self::B4800,
            3 => Self::B9600,
            4 => Self::B19200,
            5 => unreachable!("Factory reset"),
            _ => unreachable!(),
        }
    }

    /// Encodes a `BaudRate` into a `Word` protocol value for writing to a register.
    ///
    /// # Returns
    ///
    /// The corresponding `Word` containing the encoded value.
    pub fn encode_for_write_register(&self) -> Word {
        match self {
            Self::B1200 => 0u16,
            Self::B2400 => 1,
            Self::B4800 => 2,
            Self::B9600 => 3,
            Self::B19200 => 4,
        }
    }
}

/// Error indicating an invalid baud rate value
#[derive(Error, Debug)]
#[error("Baud rate must by any value of 1200, 2400, 4800, 9600, 19200.")]
pub struct ErrorInvalidBaudRate;

impl TryFrom<u16> for BaudRate {
    type Error = ErrorInvalidBaudRate;

    /// Attempts to convert a `u16` value into a `BaudRate`.
    /// Returns an error if the value does not match a valid baud rate.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1200 => Ok(BaudRate::B1200),
            2400 => Ok(BaudRate::B2400),
            4800 => Ok(BaudRate::B4800),
            9600 => Ok(BaudRate::B9600),
            19200 => Ok(BaudRate::B19200),
            _ => Err(ErrorInvalidBaudRate),
        }
    }
}

impl From<&BaudRate> for u16 {
    /// Converts a `BaudRate` reference into its corresponding `u16` representation.
    fn from(baud_rate: &BaudRate) -> u16 {
        match baud_rate {
            BaudRate::B1200 => 1200,
            BaudRate::B2400 => 2400,
            BaudRate::B4800 => 4800,
            BaudRate::B9600 => 9600,
            BaudRate::B19200 => 19200,
        }
    }
}

impl std::fmt::Display for BaudRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u16::from(self))
    }
}

/// Modbus RTU address for the RS485 serial port.
/// The address must be in the range from 1 to 247.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Address(u8);

impl std::ops::Deref for Address {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Address {
    /// Returns the factory default address for the R4DCB08 temperature module.
    fn default() -> Self {
        Self(0x01)
    }
}

impl Address {
    /// Register address for reading and writing the the Modbus device address.
    pub const ADDRESS: u16 = 0x00FE;
    /// Number of registers required to read the address.
    pub const QUANTITY: u16 = 2;
    /// Minimum valid Modbus device address.
    pub const MIN: u8 = 1;
    /// Maximum valid Modbus device address.
    pub const MAX: u8 = 247;
    /// Broadcast address for reading Modbus address (not assignable).
    pub const BROADCAST: Address = Address(0xFF);

    /// Decodes an `Address` from a Modbus holding register value.
    ///
    /// # Arguments
    ///
    /// * `words` - A slice of `Word` containing the register values.
    ///
    /// # Returns
    ///
    /// An `Address` struct containing the decoded value.
    pub fn decode_from_holding_registers(words: &[Word]) -> Self {
        Self(*words.first().unwrap() as u8)
    }

    /// Encodes an `Address` into a `Word` value for writing to a register.
    ///
    /// # Returns
    ///
    /// The corresponding `Word` containing the encoded value.
    pub fn encode_for_write_register(&self) -> Word {
        self.0 as u16
    }
}

/// Error indicating that the address value is out of range.
///
/// # Arguments
///
/// * `0` - The address value that caused the error.
#[derive(Error, Debug)]
#[error(
    "The address value {0} is outside the valid range of {min} to {max}",
    min = Address::MIN,
    max = Address::MAX
)]
pub struct ErrorAddressOutOfRange(u8);

impl TryFrom<u8> for Address {
    type Error = ErrorAddressOutOfRange;

    /// Attempts to convert a `u8` into an `Address`.
    /// Returns an error if the value is outside the valid range.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ErrorAddressOutOfRange(value))
        }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#04x}", self.0)
    }
}

/// Structure representing a temperature value.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Temperature(f32);

impl Temperature {
    /// Constant representing a NaN temperature.
    pub const NAN: Temperature = Self(f32::NAN);
    /// Minimum valid temperature value in degrees Celsius.
    pub const MIN: f32 = -3276.7;
    /// Maximum valid temperature value in degrees Celsius.
    pub const MAX: f32 = 3276.7;

    /// Decodes a `Temperature` from a Modbus holding register value.
    ///
    /// # Arguments
    ///
    /// * `word` - A `Word` containing the register values.
    ///
    /// # Returns
    ///
    /// A `Temperature` struct containing the decoded value.
    pub fn decode_from_holding_registers(word: Word) -> Self {
        Self(match word.cmp(&0x8000) {
            std::cmp::Ordering::Greater => {
                // The highest bit 1 indicates a negative value,
                // this value directly subtracting 65536 and divided by 10,
                // is the current temperature value.
                ((word as f32) - 65536.0) / 10.0
            }
            std::cmp::Ordering::Less => {
                // The highest bit is 0, so the temperature is positive,
                // it is converted to decimal and divided by 10
                word as f32 / 10.0
            }
            std::cmp::Ordering::Equal => {
                // When the data is 0X8000 (32768), it indicates no sensor or error
                f32::NAN
            }
        })
    }

    /// Encodes a `Temperature` into a `Word` value for writing to a register.
    ///
    /// # Returns
    ///
    /// The corresponding `Word` containing the encoded value.
    pub fn encode_for_write_register(&self) -> Word {
        if self.0 >= 0.0 {
            (self.0 * 10.0) as u16
        } else {
            (65536.0 + self.0 * 10.0) as u16
        }
    }
}

impl std::ops::Deref for Temperature {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Error indicating that the degree Celsius value is out of range.
///
/// # Arguments
///
/// * `0` - The degree Celsius value that caused the error.
#[derive(Error, Debug)]
#[error(
    "The degree celsius value {0} is outside the valid range of {min} to {max}",
    min = Temperature::MIN,
    max = Temperature::MAX
)]
pub struct ErrorDegreeCelsiusOutOfRange(f32);

impl TryFrom<f32> for Temperature {
    type Error = ErrorDegreeCelsiusOutOfRange;

    /// Attempts to convert a `f32` into an `Temperature`.
    /// Returns an error if the value is outside the valid range.
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            Err(ErrorDegreeCelsiusOutOfRange(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}", self.0)
    }
}

/// Structure representing a collection of temperatures.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Temperatures([Temperature; NUMBER_OF_CHANNELS]);

impl Temperatures {
    /// Register address for reading and writing the temperatures.
    pub const ADDRESS: u16 = 0x0000;
    /// Number of registers required to read the temperatures.
    pub const QUANTITY: u16 = NUMBER_OF_CHANNELS as u16;

    /// Decodes a `Temperatures` from a Modbus holding register value.
    ///
    /// # Arguments
    ///
    /// * `words` - A slice of `Word` containing the register values.
    ///
    /// # Returns
    ///
    /// A `Temperatures` struct containing the decoded value.
    pub fn decode_from_holding_registers(words: &[Word]) -> Self {
        let mut temperatures = [Temperature::NAN; NUMBER_OF_CHANNELS];
        for (i, word) in words.iter().enumerate() {
            temperatures[i] = Temperature::decode_from_holding_registers(*word);
        }
        Self(temperatures)
    }

    /// Returns an iterator over the temperatures.
    pub fn iter(&self) -> std::slice::Iter<'_, Temperature> {
        self.0.iter()
    }
}

impl std::fmt::Display for Temperatures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

/// Structure representing a channel.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Channel(u8);

impl Channel {
    /// Minimum valid channel value.
    pub const MIN: usize = 0;
    /// Maximum valid channel value.
    pub const MAX: usize = NUMBER_OF_CHANNELS - 1;
}

impl std::ops::Deref for Channel {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Error indicating that the channel value is out of range.
///
/// # Arguments
///
/// * `0` - The channel value that caused the error.
#[derive(Error, Debug)]
#[error(
    "The channel value {0} is outside the valid range of {min} to {max}",
    min = Channel::MIN,
    max = Channel::MAX
)]
pub struct ErrorChannelOutOfRange(u8);

impl TryFrom<u8> for Channel {
    type Error = ErrorChannelOutOfRange;

    /// Attempts to convert a `u8` into an `Channel`.
    /// Returns an error if the value is outside the valid range.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(Self::MIN..=Self::MAX).contains(&(value as usize)) {
            Err(ErrorChannelOutOfRange(value))
        } else {
            Ok(Self(value))
        }
    }
}

/// Structure representing temperature correction values.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemperatureCorrection([Temperature; NUMBER_OF_CHANNELS]);

impl TemperatureCorrection {
    /// Register address for reading and writing the temperature correction values.
    pub const ADDRESS: u16 = 0x0008;
    /// Number of registers required to read the temperature correction values.
    pub const QUANTITY: u16 = NUMBER_OF_CHANNELS as u16;

    /// Decodes a `TemperatureCorrection` from a Modbus holding register value.
    ///
    /// # Arguments
    ///
    /// * `words` - A slice of `Word` containing the register values.
    ///
    /// # Returns
    ///
    /// A `TemperatureCorrection` struct containing the decoded value.
    pub fn decode_from_holding_registers(words: &[Word]) -> Self {
        let mut temperatures = [Temperature::NAN; NUMBER_OF_CHANNELS];
        for (i, word) in words.iter().enumerate() {
            temperatures[i] = Temperature::decode_from_holding_registers(*word);
        }
        Self(temperatures)
    }

    /// Returns an iterator over the temperature correction values.
    pub fn iter(&self) -> std::slice::Iter<'_, Temperature> {
        self.0.iter()
    }

    /// Encodes a `Temperature` correction value into a `Word` value for writing to a register.
    ///
    /// # Returns
    ///
    /// The corresponding `Word` containing the encoded value.
    pub fn encode_for_write_register(correction_value: Temperature) -> Word {
        correction_value.encode_for_write_register()
    }

    /// Returns the register address for a specific channel.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel for which to get the address.
    ///
    /// # Returns
    ///
    /// A `u16` containing the register address for the channel.
    pub fn channel_address(channel: Channel) -> u16 {
        Self::ADDRESS + *channel as u16
    }
}

impl std::fmt::Display for TemperatureCorrection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

/// Structure representing the automatic report interval.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AutomaticReport(u8);

impl AutomaticReport {
    /// Register address for reading and writing the automatic report interval.
    pub const ADDRESS: u16 = 0x00FD;
    /// Number of registers required to read the automatic report interval.
    pub const QUANTITY: u16 = 1;
    /// Minimum valid duration in seconds for automatic reporting interval.
    pub const DURATION_MIN: u8 = 0;
    /// Maximum valid duration in seconds for automatic reporting interval.
    pub const DURATION_MAX: u8 = 255;

    /// Decodes an `AutomaticReport` from a Modbus holding register value.
    ///
    /// # Arguments
    ///
    /// * `words` - A slice of `Word` containing the register values.
    ///
    /// # Returns
    ///
    /// An `AutomaticReport` struct containing the decoded value.
    pub fn decode_from_holding_registers(words: &[Word]) -> Self {
        Self(*words.first().unwrap() as u8)
    }

    /// Encodes a `AutomaticReport` interval into a `Word` value for writing to a register.
    ///
    /// # Returns
    ///
    /// The corresponding `Word` containing the encoded value.
    pub fn encode_for_write_register(&self) -> Word {
        u16::from(self.0)
    }
}

impl std::ops::Deref for AutomaticReport {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u8> for AutomaticReport {
    fn from(interval: u8) -> AutomaticReport {
        Self(interval)
    }
}

/// Error indicating that the duration value is out of range.
///
/// # Arguments
///
/// * `0` - The duration value that caused the error.
#[derive(Error, Debug)]
#[error(
    "The duration value {0} is outside the valid range of {min} to {max}",
    min = AutomaticReport::DURATION_MIN,
    max = AutomaticReport::DURATION_MAX
)]
pub struct ErrorDurationOutOfRange(u64);

impl TryFrom<Duration> for AutomaticReport {
    type Error = ErrorDurationOutOfRange;

    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        let secs = value.as_secs();
        if (Self::DURATION_MIN as u64..=Self::DURATION_MAX as u64).contains(&secs) {
            Ok(Self(secs as u8))
        } else {
            Err(ErrorDurationOutOfRange(secs))
        }
    }
}

/// Structure representing a factory reset operation.
pub struct FactoryReset;

impl FactoryReset {
    /// Register address for writing a factory reset.
    pub const ADDRESS: u16 = 0x00FF;
    /// Data value for performing a factory reset.
    const DATA: u16 = 5;

    /// Encodes the value for performing a factory reset.
    ///
    /// # Returns
    ///
    /// The corresponding `Word` containing the encoded value.
    pub fn encode_for_write_register() -> Word {
        Self::DATA
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn degree_celsius_test() {
        assert_eq!(*Temperature::decode_from_holding_registers(219), 21.9);
        assert!(matches!(
            Temperature::try_from(21.9)
                .unwrap()
                .encode_for_write_register(),
            219
        ));

        assert_eq!(*Temperature::decode_from_holding_registers(65424), -11.2);
        assert!(matches!(
            Temperature::try_from(-11.2)
                .unwrap()
                .encode_for_write_register(),
            65424
        ));

        assert_eq!(*Temperature::decode_from_holding_registers(100), 10.0);
        assert!(matches!(
            Temperature::try_from(10.0)
                .unwrap()
                .encode_for_write_register(),
            100
        ));

        assert_eq!(*Temperature::decode_from_holding_registers(65506), -3.0);
        assert!(matches!(
            Temperature::try_from(-3.0)
                .unwrap()
                .encode_for_write_register(),
            65506
        ));

        // maximum
        assert_eq!(
            *Temperature::decode_from_holding_registers(32767),
            Temperature::MAX
        );
        assert!(matches!(
            Temperature::try_from(Temperature::MAX)
                .unwrap()
                .encode_for_write_register(),
            32767
        ));
        assert!(matches!(
            Temperature::try_from(Temperature::MAX + 0.1),
            Err(ErrorDegreeCelsiusOutOfRange(..))
        ));

        // minimum
        assert_eq!(
            *Temperature::decode_from_holding_registers(32769),
            Temperature::MIN
        );
        assert!(matches!(
            Temperature::try_from(Temperature::MIN)
                .unwrap()
                .encode_for_write_register(),
            32769
        ));
        assert!(matches!(
            Temperature::try_from(Temperature::MIN + -0.1),
            Err(ErrorDegreeCelsiusOutOfRange(..))
        ));

        assert!(Temperature::decode_from_holding_registers(32768).is_nan());
    }

    #[test]
    fn address_test() {
        assert!(matches!(
            Address::try_from(Address::MIN - 1),
            Err(ErrorAddressOutOfRange(..))
        ));
        assert!(matches!(Address::try_from(1), Ok(..)));
        assert!(matches!(Address::try_from(247), Ok(..)));
        assert!(matches!(
            Address::try_from(Address::MAX + 1),
            Err(ErrorAddressOutOfRange(..))
        ));
    }

    #[test]
    fn temperature_correction_check_channel_test() {
        assert!(matches!(Channel::try_from(Channel::MIN as u8), Ok(..)));
        assert!(matches!(Channel::try_from(Channel::MAX as u8), Ok(..)));
        assert!(matches!(
            Channel::try_from(Channel::MAX as u8 + 1),
            Err(ErrorChannelOutOfRange(..))
        ));
    }
}
