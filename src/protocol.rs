//! Defines data structures, constants, and protocol logic for interacting
//! with an 8-Channel temperature module using the Modbus RTU protocol, based on
//! the protocol specification document.
//!
//! Assumes standard Modbus function codes "Read Holding Registers" (0x03) and
//! "Write Single Register" (0x06) are used externally to interact with the device.

use std::time::Duration;
use thiserror::Error;

/// A comprehensive error type for all operations within the `protocol` module.
///
/// This enum consolidates errors that can occur during the decoding of Modbus
/// register data, encoding of values for writing, or validation of parameters.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// Error indicating that the data received from a Modbus read has an unexpected length.
    #[error("Invalid data length: expected {expected}, got {got}")]
    UnexpectedDataLength { expected: usize, got: usize },

    /// Error for malformed data within a register, e.g., an unexpected non-zero upper byte.
    #[error("Invalid data in register: {details}")]
    InvalidData { details: String },

    /// Error for an invalid value read from a register, e.g., an undefined baud rate code.
    #[error("Invalid value code for {entity}: {code}")]
    InvalidValueCode { entity: String, code: u16 },

    /// Error for an attempt to encode a value that is not supported, e.g., a `NaN` temperature.
    #[error("Cannot encode value: {reason}")]
    EncodeError { reason: String },
}

/// Represents a single 16-bit value stored in a Modbus register.
///
/// Modbus RTU typically operates on 16-bit registers.
pub type Word = u16;

/// Number of temperature channels available in the R4DCB08 module.
pub const NUMBER_OF_CHANNELS: usize = 8;

/// Enum representing the supported baud rates for RS485 communication.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u16)]
pub enum BaudRate {
    /// 1200 baud rate.
    B1200 = 1200, // Corresponds to register value 0.
    /// 2400 baud rate.
    B2400 = 2400, // Corresponds to register value 1.
    /// 4800 baud rate.
    B4800 = 4800, // Corresponds to register value 2.
    /// 9600 baud rate.
    #[default]
    B9600 = 9600, // Factory default rate. Corresponds to register value 3.
    /// 19200 baud rate.
    B19200 = 19200, // Corresponds to register value 4.
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for BaudRate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u16::deserialize(deserializer)?;
        BaudRate::try_from(value).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl BaudRate {
    /// Modbus register address for reading/writing the baud rate.
    pub const ADDRESS: u16 = 0x00FF;
    /// Number of Modbus registers holding the baud rate value.
    pub const QUANTITY: u16 = 1;

    /// Decodes a `BaudRate` from a Modbus holding register value.
    ///
    /// Reads the first word from the provided slice, which should contain the
    /// raw value read from the device's baud rate register (0x00FF).
    ///
    /// # Arguments
    ///
    /// * `words` - A slice containing the `Word`(s) read from the register.
    ///
    /// # Returns
    ///
    /// The corresponding `BaudRate` or an `Error` if decoding fails.
    ///
    /// # Errors
    ///
    /// * [`Error::UnexpectedDataLength`]: if `words` does not contain exactly one element.
    /// * [`Error::InvalidValueCode`]: if the value from the register is not a valid baud rate code.
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        if words.len() != Self::QUANTITY as usize {
            return Err(Error::UnexpectedDataLength {
                expected: Self::QUANTITY as usize,
                got: words.len(),
            });
        }
        match words[0] {
            0 => Ok(Self::B1200),
            1 => Ok(Self::B2400),
            2 => Ok(Self::B4800),
            3 => Ok(Self::B9600),
            4 => Ok(Self::B19200),
            invalid_code => Err(Error::InvalidValueCode {
                entity: "BaudRate".to_string(),
                code: invalid_code,
            }),
        }
    }

    /// Encodes the `BaudRate` into its corresponding `Word` value for writing to the Modbus register.
    ///
    /// # Returns
    ///
    /// The `Word` representation of the baud rate.
    pub fn encode_for_write_register(&self) -> Word {
        match self {
            Self::B1200 => 0,
            Self::B2400 => 1,
            Self::B4800 => 2,
            Self::B9600 => 3,
            Self::B19200 => 4,
        }
    }
}

/// Error returned when attempting to create a `BaudRate` from an unsupported `u16` value.
#[derive(Error, Debug, PartialEq, Eq)]
#[error("Unsupported baud rate value: {0}. Must be 1200, 2400, 4800, 9600, or 19200.")]
pub struct ErrorInvalidBaudRate(
    /// The invalid baud rate value that caused the error.
    pub u16,
);

impl TryFrom<u16> for BaudRate {
    type Error = ErrorInvalidBaudRate;

    /// Attempts to convert a standard `u16` baud rate value (e.g., 9600) into a `BaudRate` enum variant.
    /// Returns an error if the value does not match a supported baud rate.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1200 => Ok(BaudRate::B1200),
            2400 => Ok(BaudRate::B2400),
            4800 => Ok(BaudRate::B4800),
            9600 => Ok(BaudRate::B9600),
            19200 => Ok(BaudRate::B19200),
            _ => Err(ErrorInvalidBaudRate(value)),
        }
    }
}

impl From<BaudRate> for u16 {
    /// Converts a `BaudRate` variant into its standard `u16` representation (e.g., `BaudRate::B9600` becomes `9600`).
    fn from(baud_rate: BaudRate) -> u16 {
        match baud_rate {
            BaudRate::B1200 => 1200,
            BaudRate::B2400 => 2400,
            BaudRate::B4800 => 4800,
            BaudRate::B9600 => 9600,
            BaudRate::B19200 => 19200,
        }
    }
}

impl From<BaudRate> for u32 {
    fn from(baud_rate: BaudRate) -> u32 {
        baud_rate as u16 as u32
    }
}

impl std::fmt::Display for BaudRate {
    /// Formats the `BaudRate` as its numeric value (e.g., "9600").
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u16)
    }
}

/// Represents a validated Modbus device address, used for RTU communication over RS485.
///
/// Valid addresses are in the range 1 to 247 (inclusive).
/// Use [`Address::try_from`] to create an instance from a `u8`.
/// Provides constants and methods for reading/writing the device address itself via Modbus commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Address(u8);

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Address::try_from(value).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

/// Allows direct access to the underlying `u8` address value.
impl std::ops::Deref for Address {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Address {
    /// Returns the factory default Modbus address.
    fn default() -> Self {
        // Safe because `0x01` is within the valid range 1-247.
        Self(0x01)
    }
}

impl Address {
    /// The Modbus register address used to read or write the device's own Modbus address.
    ///
    /// Use Modbus function 0x03 (Read Holding Registers) to read (requires addressing
    /// the device with its *current* address or the broadcast address).
    /// Use function 0x06 (Write Single Register) to change the address (also requires
    /// addressing the device with its current address).
    pub const ADDRESS: u16 = 0x00FE;

    /// The number of registers to read when reading the device address.
    pub const QUANTITY: u16 = 1; // The protocol specification indicates 2 bytes but the documentation accurately reflects that it expects a slice of length 1

    /// The minimum valid assignable Modbus device address (inclusive).
    pub const MIN: u8 = 1;
    /// The maximum valid assignable Modbus device address (inclusive).
    pub const MAX: u8 = 247;

    /// The Modbus broadcast address (`0xFF` or 255).
    ///
    /// Can be used for reading the device address when it's unknown.
    /// This address cannot be assigned to a device as its permanent address.
    pub const BROADCAST: Address = Address(0xFF);

    /// Decodes the device [`Address`] from a Modbus holding register value read from the device.
    ///
    /// Expects `words` to contain the single register value read from the device address
    /// configuration register ([`Address::ADDRESS`]).
    /// It validates that the decoded address is within the assignable range ([`Address::MIN`]..=[`Address::MAX`]).
    ///
    /// # Arguments
    ///
    /// * `words`: A slice containing the [`Word`] value read from the device address register. Expected to have length 1.
    ///
    /// # Returns
    ///
    /// An [`Address`] struct containing the decoded and validated value, or an `Error` if decoding fails.
    ///
    /// # Errors
    ///
    /// * [`Error::UnexpectedDataLength`]: if `words` does not contain exactly one element.
    /// * [`Error::InvalidData`]: if the upper byte of the register value is non-zero.
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        if words.len() != Self::QUANTITY as usize {
            return Err(Error::UnexpectedDataLength {
                expected: Self::QUANTITY as usize,
                got: words.len(),
            });
        }
        let word_value = words[0];

        if word_value & 0xFF00 != 0 {
            return Err(Error::InvalidData {
                details: format!(
                    "Upper byte of address register is non-zero (value: {word_value:#06X})"
                ),
            });
        }

        let address_byte = word_value as u8;
        match Self::try_from(address_byte) {
            Ok(address) => Ok(address),
            Err(err) => Err(Error::InvalidData {
                details: format!("{err}"),
            }),
        }
    }

    /// Encodes the `Address` into its `Word` representation for writing to the Modbus register.
    ///
    /// # Returns
    ///
    /// The `Word` representation of the address.
    pub fn encode_for_write_register(&self) -> Word {
        self.0 as Word
    }
}

/// Error indicating that a provided Modbus device address (`u8`) is outside the valid range
/// for assignable addresses, defined by [`Address::MIN`] and [`Address::MAX`] (inclusive).
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
#[error(
    "The address value {0} is outside the valid assignable range of {min} to {max}",
    min = Address::MIN,
    max = Address::MAX
)]
pub struct ErrorAddressOutOfRange(
    /// The invalid address value that caused the error.
    pub u8,
);

impl TryFrom<u8> for Address {
    type Error = ErrorAddressOutOfRange;

    /// Attempts to create an [`Address`] from a `u8` value, validating its assignable range [[`Address::MIN`], [`Address::MAX`]].
    ///
    /// # Arguments
    ///
    /// * `value`: The Modbus address to validate.
    ///
    /// # Returns
    ///
    /// * `Ok(Address)`: If the `value` is within the valid assignable range [[`Address::MIN`], [`Address::MAX`]].
    /// * `Err(ErrorAddressOutOfRange)`: If the `value` is outside the valid assignable range (e.g., 0 or > 247).
    ///
    /// # Example
    /// ```
    /// # use r4dcb08_lib::protocol::{Address, ErrorAddressOutOfRange};
    /// assert!(matches!(Address::try_from(0), Err(ErrorAddressOutOfRange(_))));
    /// assert!(Address::try_from(1).is_ok());
    /// assert!(Address::try_from(247).is_ok());
    /// assert_eq!(Address::try_from(248).unwrap_err(), ErrorAddressOutOfRange(248));
    /// assert!(matches!(Address::try_from(255), Err(ErrorAddressOutOfRange(_)))); // Broadcast address is not valid for TryFrom
    /// ```
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ErrorAddressOutOfRange(value))
        }
    }
}

/// Provides a hexadecimal string representation (e.g., "0x01", "0xf7").
impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#04x}", self.0)
    }
}

/// Represents a temperature value in degrees Celsius, as read from or written to the R4DCB08.
///
/// Valid addresses are in the range -3276.8 to 3276.7 (inclusive).
/// Use [`Temperature::try_from`] to create an instance from a `f32`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Temperature(f32);

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Temperature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f32::deserialize(deserializer)?;
        // Allow NAN during deserialization, but TryFrom will catch out-of-range non-NAN values.
        if value.is_nan() {
            Ok(Temperature::NAN)
        } else {
            Temperature::try_from(value).map_err(|e| serde::de::Error::custom(e.to_string()))
        }
    }
}

impl Temperature {
    /// Represents a Not-a-Number temperature, often indicating a sensor error or disconnection.
    pub const NAN: Temperature = Self(f32::NAN); // Corresponds to the register value 0x8000 (32768)
    /// Minimum measurable/representable temperature value (-3276.7 °C).
    pub const MIN: f32 = -3276.7; // Derived from the most negative signed 16-bit integer divided by 10.
    /// Maximum measurable/representable temperature value (+3276.7 °C).
    pub const MAX: f32 = 3276.7; // Derived from the most positive signed 16-bit integer divided by 10.

    /// Decodes a `Temperature` from a single Modbus holding register `Word`.
    ///
    /// # Arguments
    ///
    /// * `word` - The `Word` value read from the temperature or correction register.
    ///
    /// # Returns
    ///
    /// The decoded `Temperature`. Returns `Temperature::NAN` indicating a sensor error or disconnection.
    pub fn decode_from_holding_registers(word: Word) -> Self {
        // Implements the decoding logic described in the protocol:
        // - Value `0x8000` (32768) represents NAN (no sensor/error).
        // - If the highest bit is 1 (value > 0x8000), it's negative: `(value - 65536) / 10.0`.
        // - If the highest bit is 0 (value < 0x8000), it's positive: `value / 10.0`.
        match word {
            0x8000 => Self::NAN,
            w if w > 0x8000 => Self(((w as f32) - 65536.0) / 10.0),
            w => Self((w as f32) / 10.0),
        }
    }

    /// Encodes the `Temperature` into a `Word` value for writing to a Modbus register.
    ///
    /// # Returns
    ///
    /// The `Word` representation of the temperature, ready for writing, or an `Error` if encoding fails.
    ///
    /// # Errors
    ///
    /// * [`Error::EncodeError`]: if the temperature value is `NAN`.
    pub fn encode_for_write_register(&self) -> Result<Word, Error> {
        // Implements the encoding logic derived from protocol examples:
        // - Positive values are multiplied by 10.
        // - Negative values are encoded using two's complement representation after multiplying by 10.
        //   `(value * 10.0) as i16 as u16` achieves this.
        // - NAN cannot be directly encoded; attempting to encode NAN will result in an undefined `Word` value.
        if self.0.is_nan() {
            return Err(Error::EncodeError {
                reason: "Temperature value is NAN, which cannot be encoded.".to_string(),
            });
        }
        if self.0 >= 0.0 {
            Ok((self.0 * 10.0) as Word)
        } else {
            Ok((65536.0 + self.0 * 10.0) as Word)
        }
    }
}

/// Allows direct access to the underlying `f32` temperature value.
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
#[derive(Error, Debug, PartialEq)]
#[error(
    "The degree celsius value {0} is outside the valid range of {min} to {max}",
    min = Temperature::MIN,
    max = Temperature::MAX
)]
pub struct ErrorDegreeCelsiusOutOfRange(
    /// The invalid degree celsius value that caused the error.
    pub f32,
);

impl TryFrom<f32> for Temperature {
    type Error = ErrorDegreeCelsiusOutOfRange;

    /// Attempts to create a `Temperature` from an `f32` value.
    /// Returns an error if the value is outside the representable range [`MIN`, `MAX`].
    /// Note: This does **not** allow creating `Temperature::NAN` via `try_from`.
    /// Use `Temperature::NAN` constant directly.
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_nan() {
            // Explicitly disallow creating NAN via try_from, require using the constant.
            Err(ErrorDegreeCelsiusOutOfRange(value))
        } else if !(Self::MIN..=Self::MAX).contains(&value) {
            Err(ErrorDegreeCelsiusOutOfRange(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl std::fmt::Display for Temperature {
    /// Formats the `Temperature` to one decimal place (e.g., "21.9", "-3.0", "NAN").
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_nan() {
            write!(f, "NAN")
        } else {
            write!(f, "{:.1}", self.0)
        }
    }
}

/// Represents the temperature readings for all 8 channels.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Temperatures([Temperature; NUMBER_OF_CHANNELS]);

impl Temperatures {
    /// Register address for reading and writing the temperatures.
    pub const ADDRESS: u16 = 0x0000;
    /// Number of Modbus registers required to read all channel temperatures.
    pub const QUANTITY: u16 = NUMBER_OF_CHANNELS as u16;

    /// Decodes `Temperatures` for all channels from a slice of Modbus holding register values.
    ///
    /// Expects a slice containing `NUMBER_OF_CHANNELS` words.
    ///
    /// # Arguments
    ///
    /// * `words` - A slice of `Word` containing the register values for all channels.
    ///
    /// # Returns
    ///
    /// A `Temperatures` struct containing the decoded value for each channel.
    ///
    /// # Errors
    ///
    /// * [`Error::UnexpectedDataLength`]: if `words` does not have length equal to [`NUMBER_OF_CHANNELS`].
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        if words.len() != NUMBER_OF_CHANNELS {
            return Err(Error::UnexpectedDataLength {
                expected: NUMBER_OF_CHANNELS,
                got: words.len(),
            });
        }
        let mut temperatures = [Temperature::NAN; NUMBER_OF_CHANNELS];
        for (i, word) in words.iter().enumerate() {
            temperatures[i] = Temperature::decode_from_holding_registers(*word);
        }
        Ok(Self(temperatures))
    }

    /// Returns an iterator over the individual `Temperature` values.
    pub fn iter(&self) -> std::slice::Iter<'_, Temperature> {
        self.0.iter()
    }

    /// Returns a slice containing all `Temperature` values.
    pub fn as_slice(&self) -> &[Temperature] {
        &self.0
    }

    /// Provides direct access to the underlying array of port states.
    pub fn as_array(&self) -> &[Temperature; NUMBER_OF_CHANNELS] {
        &self.0
    }
}

impl IntoIterator for Temperatures {
    type Item = Temperature;
    type IntoIter = std::array::IntoIter<Temperature, NUMBER_OF_CHANNELS>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Temperatures {
    type Item = &'a Temperature;
    type IntoIter = std::slice::Iter<'a, Temperature>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl std::fmt::Display for Temperatures {
    /// Formats the temperatures as a comma-separated string (e.g., "21.9, -11.2, ...").
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let temp_strs: Vec<String> = self.0.iter().map(|t| t.to_string()).collect();
        write!(f, "{}", temp_strs.join(", "))
    }
}

impl std::ops::Index<usize> for Temperatures {
    type Output = Temperature;

    /// Allows indexing into the temperatures array (e.g., `temps[0]` for CH1).
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds (0-7).
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

/// Represents a validated temperature channel index, guaranteed to be within the valid range `0..NUMBER_OF_CHANNELS`.
///
/// Use [`Channel::try_from`] to create an instance from a `u8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Channel(u8);

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Channel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Channel::try_from(value).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl Channel {
    /// The minimum valid channel index (inclusive).
    pub const MIN: u8 = 0;
    /// The maximum valid channel index (inclusive).
    pub const MAX: u8 = NUMBER_OF_CHANNELS as u8 - 1;
}

/// Allows direct access to the underlying `u8` channel number.
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
#[derive(Error, Debug, PartialEq, Eq)]
#[error(
    "The channel value {0} is outside the valid range of {min} to {max}",
    min = Channel::MIN,
    max = Channel::MAX
)]
pub struct ErrorChannelOutOfRange(
    /// The invalid channel value that caused the error.
    pub u8,
);

impl TryFrom<u8> for Channel {
    type Error = ErrorChannelOutOfRange;

    /// Attempts to create a `Channel` from a `u8` value.
    /// Returns an error if the value is outside the valid range [0, 7].
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ErrorChannelOutOfRange(value))
        }
    }
}

impl std::fmt::Display for Channel {
    /// Formats the channel as its number (e.g., "0", "7").
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the temperature correction values for all 8 channels.
/// Correction values are added to the raw temperature reading.
/// Setting a correction value to 0 disables correction for that channel.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemperatureCorrection([Temperature; NUMBER_OF_CHANNELS]);

impl TemperatureCorrection {
    /// Starting Modbus register address for reading/writing correction values.
    pub const ADDRESS: u16 = 0x0008;
    /// Number of Modbus registers required for all channel correction values.
    pub const QUANTITY: u16 = NUMBER_OF_CHANNELS as u16;

    /// Decodes `TemperatureCorrection` values for all channels from a slice of Modbus holding register values.
    ///
    /// Expects a slice containing [`NUMBER_OF_CHANNELS`] words.
    ///
    /// # Arguments
    ///
    /// * `words` - A slice of `Word` containing the register values for all correction channels.
    ///
    /// # Returns
    ///
    /// A `TemperatureCorrection` struct containing the decoded value.
    ///
    /// # Errors
    ///
    /// * [`Error::UnexpectedDataLength`]: if `words` does not have length equal to [`NUMBER_OF_CHANNELS`].
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        if words.len() != NUMBER_OF_CHANNELS {
            return Err(Error::UnexpectedDataLength {
                expected: NUMBER_OF_CHANNELS,
                got: words.len(),
            });
        }
        let mut corrections = [Temperature::NAN; NUMBER_OF_CHANNELS];
        for (i, word) in words.iter().enumerate() {
            // Decoding is the same as Temperature
            corrections[i] = Temperature::decode_from_holding_registers(*word);
        }
        Ok(Self(corrections))
    }

    /// Returns an iterator over the individual `Temperature` correction values.
    pub fn iter(&self) -> std::slice::Iter<'_, Temperature> {
        self.0.iter()
    }

    /// Returns a slice containing all `Temperature` correction values.
    pub fn as_slice(&self) -> &[Temperature] {
        &self.0
    }

    /// Provides direct access to the underlying array of port states.
    pub fn as_array(&self) -> &[Temperature; NUMBER_OF_CHANNELS] {
        &self.0
    }

    /// Encodes a single `Temperature` correction value into a `Word` for writing to the appropriate channel register.
    ///
    /// Use `channel_address` to determine the correct register address for writing.
    ///
    /// # Arguments
    ///
    /// * `correction_value` - The `Temperature` correction value to encode.
    ///
    /// # Returns
    ///
    /// The `Word` representation of the correction value.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::EncodeError`] if the `correction_value` is `NAN`.
    pub fn encode_for_write_register(correction_value: Temperature) -> Result<Word, Error> {
        correction_value.encode_for_write_register()
    }

    /// Calculates the Modbus register address for a specific channel's correction value.
    ///
    /// # Arguments
    ///
    /// * `channel` - The `Channel` for which to get the correction register address.
    ///
    /// # Returns
    ///
    /// The `u16` Modbus register address.
    pub fn channel_address(channel: Channel) -> u16 {
        Self::ADDRESS + (*channel as u16)
    }
}

impl IntoIterator for TemperatureCorrection {
    type Item = Temperature;
    type IntoIter = std::array::IntoIter<Temperature, NUMBER_OF_CHANNELS>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a TemperatureCorrection {
    type Item = &'a Temperature;
    type IntoIter = std::slice::Iter<'a, Temperature>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl std::fmt::Display for TemperatureCorrection {
    /// Formats the correction values as a comma-separated string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let corr_strs: Vec<String> = self.0.iter().map(|t| t.to_string()).collect();
        write!(f, "{}", corr_strs.join(", "))
    }
}

impl std::ops::Index<usize> for TemperatureCorrection {
    type Output = Temperature;
    /// Allows indexing into the corrections array (e.g., `corrections[0]` for CH1).
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds (0-7).
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

/// Represents the automatic temperature reporting interval in seconds.
/// A value of 0 disables automatic reporting (query mode).
/// Values 1-255 set the reporting interval in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AutomaticReport(u8);

impl AutomaticReport {
    /// Modbus register address for reading/writing the automatic report interval.
    pub const ADDRESS: u16 = 0x00FD;
    /// The number of registers to read when reading the interval value.
    pub const QUANTITY: u16 = 1;

    /// Minimum valid interval duration (0 seconds = disabled).
    pub const DURATION_MIN: u8 = 0;
    /// Maximum valid interval duration (255 seconds).
    pub const DURATION_MAX: u8 = 255;

    /// Disables automatic reporting. Equivalent to `AutomaticReport(0)`.
    pub const DISABLED: AutomaticReport = AutomaticReport(0);

    /// Decodes an `AutomaticReport` interval from Modbus holding register data.
    ///
    /// # Arguments
    ///
    /// * `words` - A slice of `Word` containing the register value.
    ///
    /// # Returns
    ///
    /// The decoded `AutomaticReport` struct or an `Error` if decoding fails.
    ///
    /// # Errors
    ///
    /// * [`Error::UnexpectedDataLength`]: if `words` does not contain exactly one element.
    /// * [`Error::InvalidData`]: if the upper byte of the register value is non-zero.
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        if words.len() != Self::QUANTITY as usize {
            return Err(Error::UnexpectedDataLength {
                expected: Self::QUANTITY as usize,
                got: words.len(),
            });
        }
        let word_value = words[0];

        if word_value & 0xFF00 != 0 {
            return Err(Error::InvalidData {
                details: format!(
                    "Upper byte of automatic report register is non-zero (value: {word_value:#06X})"
                ),
            });
        }

        // not  use Self::try_from() - conversion cannot fail
        Ok(Self::from(word_value as u8))
    }

    /// Encodes the `AutomaticReport` interval into a `Word` value for writing to the Modbus register.
    ///
    /// # Returns
    ///
    /// The `Word` representation (0-255) of the interval.
    pub fn encode_for_write_register(&self) -> Word {
        self.0 as Word
    }

    /// Returns the interval duration in seconds. 0 means disabled.
    pub fn as_secs(&self) -> u8 {
        self.0
    }

    /// Returns the interval as a `std::time::Duration`.
    /// Returns `Duration::ZERO` if the interval is 0 (disabled).
    pub fn as_duration(&self) -> Duration {
        Duration::from_secs(self.0 as u64)
    }

    /// Checks if automatic reporting is disabled (interval is 0).
    pub fn is_disabled(&self) -> bool {
        self.0 == Self::DURATION_MIN
    }
}

impl std::ops::Deref for AutomaticReport {
    type Target = u8;
    /// Allows direct access to the underlying `u8` interval value (seconds).
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u8> for AutomaticReport {
    /// Creates an `AutomaticReport` from a `u8` value (seconds).
    /// Assumes the value is within the valid range [0, 255].
    /// Consider using `try_from(Duration)` for validation.
    fn from(interval_seconds: u8) -> AutomaticReport {
        // No bounds check here, relies on caller providing valid value or using TryFrom
        Self(interval_seconds)
    }
}

/// Error indicating that a `Duration` cannot be represented as an `AutomaticReport` interval (0-255 seconds).
#[derive(Error, Debug, PartialEq, Eq)]
#[error(
    "The duration {0} seconds is outside the valid automatic report range [{min}, {max}] seconds",
    min = AutomaticReport::DURATION_MIN,
    max = AutomaticReport::DURATION_MAX
)]
pub struct ErrorDurationOutOfRange(
    /// The invalid duration value that caused the error.
    pub u64,
);

impl TryFrom<Duration> for AutomaticReport {
    type Error = ErrorDurationOutOfRange;

    /// Attempts to create an `AutomaticReport` interval from a `std::time::Duration`.
    /// Returns an error if the duration (in whole seconds) is outside the range [0, 255].
    /// Fractional seconds are ignored.
    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        let secs = value.as_secs();
        if (Self::DURATION_MIN as u64..=Self::DURATION_MAX as u64).contains(&secs) {
            Ok(Self(secs as u8))
        } else {
            Err(ErrorDurationOutOfRange(secs))
        }
    }
}

impl std::fmt::Display for AutomaticReport {
    /// Formats the interval (e.g., "0s", "10s", "255s").
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}s", self.0)
    }
}

/// Represents the factory reset command. This is a write-only operation.
#[derive(Debug, Clone, Copy)]
pub struct FactoryReset;

impl FactoryReset {
    /// Modbus register address used for triggering a factory reset.
    pub const ADDRESS: u16 = 0x00FF;
    /// The specific data value that must be written to `ADDRESS` to trigger a factory reset.
    const DATA: u16 = 5;

    /// Encodes the required `Word` value to trigger a factory reset when written
    /// to the appropriate register (`ADDRESS`).
    ///
    /// # Returns
    ///
    /// The constant `Word` value (5).
    pub fn encode_for_write_register() -> Word {
        Self::DATA
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    // --- Temperature Tests ---
    #[test]
    fn temperature_decode() {
        // Positive values
        assert_eq!(
            Temperature::decode_from_holding_registers(219),
            Temperature::try_from(21.9).unwrap()
        );
        assert_eq!(
            Temperature::decode_from_holding_registers(100),
            Temperature::try_from(10.0).unwrap()
        );
        assert_eq!(
            Temperature::decode_from_holding_registers(0),
            Temperature::try_from(0.0).unwrap()
        );
        // Negative values
        assert_eq!(
            Temperature::decode_from_holding_registers(65424),
            Temperature::try_from(-11.2).unwrap()
        );
        assert_eq!(
            Temperature::decode_from_holding_registers(65506),
            Temperature::try_from(-3.0).unwrap()
        );
        // Boundary Negative (Smallest negative value representable)
        assert_eq!(
            Temperature::decode_from_holding_registers(0xFFFF),
            Temperature::try_from(-0.1).unwrap()
        ); // -1 -> FF F5? No, (FFFFh as i16) = -1. So -0.1
        // Boundary Negative (Largest negative value / MIN)
        // 65536 - 32768 = 32768 = 0x8000 (NAN)
        // Smallest s16 is -32768 -> 0x8000. Encoded is -3276.8
        assert_eq!(
            Temperature::decode_from_holding_registers(0x8001),
            Temperature::try_from(-3276.7).unwrap()
        ); // -32767 -> 8001h
        assert!(Temperature::decode_from_holding_registers(i16::MIN as u16).is_nan()); // -32768 -> 8000h (conflict with NAN) - Let's test the documented limit
        assert_eq!(
            Temperature::decode_from_holding_registers(32769),
            Temperature::try_from(-3276.7).unwrap()
        ); // From original tests, assuming this was intended limit instead of -3276.8

        // Boundary Positive (Largest positive value / MAX)
        assert_eq!(
            Temperature::decode_from_holding_registers(32767),
            Temperature::try_from(3276.7).unwrap()
        );
        // NAN value
        assert!(Temperature::decode_from_holding_registers(32768).is_nan());
        assert!(Temperature::decode_from_holding_registers(0x8000).is_nan());
    }

    #[test]
    fn temperature_encode() {
        // Positive
        assert_eq!(
            Temperature::try_from(21.9)
                .unwrap()
                .encode_for_write_register(),
            Ok(219)
        );
        assert_eq!(
            Temperature::try_from(10.0)
                .unwrap()
                .encode_for_write_register(),
            Ok(100)
        );
        assert_eq!(
            Temperature::try_from(0.0)
                .unwrap()
                .encode_for_write_register(),
            Ok(0)
        );
        // Negative
        assert_eq!(
            Temperature::try_from(-11.2)
                .unwrap()
                .encode_for_write_register(),
            Ok((-11.2 * 10.0) as i16 as u16)
        ); // 65424
        assert_eq!(
            Temperature::try_from(-3.0)
                .unwrap()
                .encode_for_write_register(),
            Ok((-3.0 * 10.0) as i16 as u16)
        ); // 65506
        assert_eq!(
            Temperature::try_from(-0.1)
                .unwrap()
                .encode_for_write_register(),
            Ok((-0.1 * 10.0) as i16 as u16)
        ); // 0xFFFF

        // Boundaries
        assert_eq!(
            Temperature::try_from(Temperature::MAX)
                .unwrap()
                .encode_for_write_register(),
            Ok(32767)
        );
        assert_eq!(
            Temperature::try_from(Temperature::MIN)
                .unwrap()
                .encode_for_write_register(),
            Ok((Temperature::MIN * 10.0) as i16 as u16)
        ); // -32767 -> 0x8001
    }

    #[test]
    fn temperature_encode_nan() {
        // NAN
        // Asserting specific value might be brittle, depends on desired handling.
        // Here we assume it encodes to 0x8000 as per our implementation note.
        assert_matches!(Temperature::NAN.encode_for_write_register(), Err(..));
    }

    #[test]
    fn temperature_try_from() {
        // Valid
        assert_matches!(Temperature::try_from(21.9), Ok(t) if *t == 21.9);
        assert_matches!(Temperature::try_from(-11.2), Ok(t) if *t == -11.2);
        assert_matches!(Temperature::try_from(Temperature::MAX), Ok(t) if *t == Temperature::MAX);
        assert_matches!(Temperature::try_from(Temperature::MIN), Ok(t) if *t == Temperature::MIN);

        // Invalid (Out of Range)
        let max_err_val = Temperature::MAX + 0.1;
        assert_matches!(Temperature::try_from(max_err_val), Err(ErrorDegreeCelsiusOutOfRange(value)) if value == max_err_val);

        let min_err_val = Temperature::MIN - 0.1;
        assert_matches!(Temperature::try_from(min_err_val), Err(ErrorDegreeCelsiusOutOfRange(value)) if value == min_err_val);

        assert_matches!(
            Temperature::try_from(f32::NAN),
            Err(ErrorDegreeCelsiusOutOfRange(..))
        );
    }

    #[test]
    fn temperature_display() {
        assert_eq!(Temperature::try_from(21.9).unwrap().to_string(), "21.9");
        assert_eq!(Temperature::try_from(-3.0).unwrap().to_string(), "-3.0");
        assert_eq!(Temperature::try_from(0.0).unwrap().to_string(), "0.0");
        assert_eq!(Temperature::NAN.to_string(), "NAN");
    }

    // --- Address Tests ---
    #[test]
    fn address_try_from() {
        assert_matches!(Address::try_from(0), Err(ErrorAddressOutOfRange(0)));
        assert_matches!(Address::try_from(Address::MIN), Ok(a) if *a == Address::MIN);
        assert_matches!(Address::try_from(100), Ok(a) if *a == 100);
        assert_matches!(Address::try_from(Address::MAX), Ok(a) if *a == Address::MAX);
        assert_matches!(
            Address::try_from(Address::MAX + 1),
            Err(ErrorAddressOutOfRange(248))
        );
        assert_matches!(Address::try_from(255), Err(ErrorAddressOutOfRange(255))); // Broadcast address is not valid via TryFrom
    }

    #[test]
    fn address_decode() {
        assert_eq!(
            Address::decode_from_holding_registers(&[0x0001]).unwrap(),
            Address::try_from(1).unwrap()
        );
        assert_eq!(
            Address::decode_from_holding_registers(&[0x00F7]).unwrap(),
            Address::try_from(247).unwrap()
        );
        // Example from protocol returns 0x0001 for address 1
        assert_eq!(
            Address::decode_from_holding_registers(&[0x0001]).unwrap(),
            Address::try_from(1).unwrap()
        );
    }

    #[test]
    fn address_decode_empty() {
        assert_matches!(Address::decode_from_holding_registers(&[]), Err(..));
    }

    #[test]
    fn address_decode_invalid() {
        assert_matches!(Address::decode_from_holding_registers(&[0x0000]), Err(..)); // Address 0 is invalid
    }

    #[test]
    fn address_encode() {
        assert_eq!(
            Address::try_from(1).unwrap().encode_for_write_register(),
            0x0001
        );
        assert_eq!(
            Address::try_from(247).unwrap().encode_for_write_register(),
            0x00F7
        );
        assert_eq!(Address::default().encode_for_write_register(), 0x0001);
    }

    #[test]
    fn address_display() {
        assert_eq!(Address::try_from(1).unwrap().to_string(), "0x01");
        assert_eq!(Address::try_from(25).unwrap().to_string(), "0x19");
        assert_eq!(Address::try_from(247).unwrap().to_string(), "0xf7");
    }

    #[test]
    fn address_constants() {
        assert_eq!(*Address::BROADCAST, 0xFF);
    }

    // --- BaudRate Tests ---
    #[test]
    fn baudrate_decode() {
        assert_matches!(
            BaudRate::decode_from_holding_registers(&[0]).unwrap(),
            BaudRate::B1200
        );
        assert_matches!(
            BaudRate::decode_from_holding_registers(&[1]).unwrap(),
            BaudRate::B2400
        );
        assert_matches!(
            BaudRate::decode_from_holding_registers(&[2]).unwrap(),
            BaudRate::B4800
        );
        assert_matches!(
            BaudRate::decode_from_holding_registers(&[3]).unwrap(),
            BaudRate::B9600
        );
        assert_matches!(
            BaudRate::decode_from_holding_registers(&[4]).unwrap(),
            BaudRate::B19200
        );
    }

    #[test]
    fn baudrate_decode_panics_on_invalid_value_high1() {
        assert_matches!(BaudRate::decode_from_holding_registers(&[5]), Err(..));
    }

    #[test]
    fn baudrate_decode_panics_on_invalid_value_zero() {
        assert_matches!(BaudRate::decode_from_holding_registers(&[]), Err(..));
    }

    #[test]
    fn baudrate_encode() {
        assert_eq!(BaudRate::B1200.encode_for_write_register(), 0);
        assert_eq!(BaudRate::B2400.encode_for_write_register(), 1);
        assert_eq!(BaudRate::B4800.encode_for_write_register(), 2);
        assert_eq!(BaudRate::B9600.encode_for_write_register(), 3);
        assert_eq!(BaudRate::B19200.encode_for_write_register(), 4);
        assert_eq!(BaudRate::default().encode_for_write_register(), 3);
    }

    #[test]
    fn baudrate_try_from_u16() {
        assert_matches!(BaudRate::try_from(1200), Ok(BaudRate::B1200));
        assert_matches!(BaudRate::try_from(2400), Ok(BaudRate::B2400));
        assert_matches!(BaudRate::try_from(4800), Ok(BaudRate::B4800));
        assert_matches!(BaudRate::try_from(9600), Ok(BaudRate::B9600));
        assert_matches!(BaudRate::try_from(19200), Ok(BaudRate::B19200));
        // Invalid values
        assert_matches!(BaudRate::try_from(0), Err(ErrorInvalidBaudRate(0)));
        assert_matches!(BaudRate::try_from(1000), Err(ErrorInvalidBaudRate(1000)));
        assert_matches!(BaudRate::try_from(38400), Err(ErrorInvalidBaudRate(38400)));
    }

    #[test]
    fn baudrate_from_baudrate_to_u16() {
        assert_eq!(u16::from(BaudRate::B1200), 1200);
        assert_eq!(u16::from(BaudRate::B2400), 2400);
        assert_eq!(u16::from(BaudRate::B4800), 4800);
        assert_eq!(u16::from(BaudRate::B9600), 9600);
        assert_eq!(u16::from(BaudRate::B19200), 19200);
        assert_eq!(u16::from(BaudRate::default()), 9600);
    }

    #[test]
    fn baudrate_display() {
        assert_eq!(BaudRate::B1200.to_string(), "1200");
        assert_eq!(BaudRate::B9600.to_string(), "9600");
        assert_eq!(BaudRate::B19200.to_string(), "19200");
        assert_eq!(BaudRate::default().to_string(), "9600");
    }

    // --- Channel Tests ---
    #[test]
    fn channel_try_from() {
        assert_matches!(Channel::try_from(Channel::MIN), Ok(c) if *c == Channel::MIN);
        assert_matches!(Channel::try_from(3), Ok(c) if *c == 3);
        assert_matches!(Channel::try_from(Channel::MAX), Ok(c) if *c == Channel::MAX);
        // Invalid
        // MIN is 0, so no lower bound test needed for u8
        assert_matches!(
            Channel::try_from(Channel::MAX + 1),
            Err(ErrorChannelOutOfRange(8))
        );
        assert_matches!(Channel::try_from(255), Err(ErrorChannelOutOfRange(255)));
    }

    #[test]
    fn channel_new() {
        assert_eq!(*Channel::try_from(0).unwrap(), 0);
        assert_eq!(*Channel::try_from(7).unwrap(), 7);
    }

    #[test]
    fn channel_display() {
        assert_eq!(Channel::try_from(0).unwrap().to_string(), "0");
        assert_eq!(Channel::try_from(7).unwrap().to_string(), "7");
    }

    // --- Temperatures Tests ---
    #[test]
    fn temperatures_decode() {
        let words = [219, 65424, 100, 65506, 32767, 32769, 0, 32768]; // Mix of temps + NAN
        let temps = Temperatures::decode_from_holding_registers(&words).unwrap();

        assert_eq!(temps[0], Temperature::try_from(21.9).unwrap());
        assert_eq!(temps[1], Temperature::try_from(-11.2).unwrap());
        assert_eq!(temps[2], Temperature::try_from(10.0).unwrap());
        assert_eq!(temps[3], Temperature::try_from(-3.0).unwrap());
        assert_eq!(temps[4], Temperature::try_from(3276.7).unwrap()); // MAX
        assert_eq!(temps[5], Temperature::try_from(-3276.7).unwrap()); // MIN
        assert_eq!(temps[6], Temperature::try_from(0.0).unwrap());
        assert!(temps[7].is_nan()); // NAN

        // Check iterator
        let mut iter = temps.iter();
        assert_eq!(iter.next(), Some(&Temperature::try_from(21.9).unwrap()));
        assert_eq!(iter.next(), Some(&Temperature::try_from(-11.2).unwrap()));
        // ... (check others if desired)
        assert!(iter.nth(5).expect("7th element (index 7)").is_nan());
        assert_eq!(iter.next(), None);

        // Check indexing
        assert_eq!(temps[0], Temperature::try_from(21.9).unwrap());
        assert!(temps[7].is_nan());
    }

    #[test]
    fn temperatures_decode_wrong_size() {
        let words = [219, 65424]; // Too few
        assert_matches!(Temperatures::decode_from_holding_registers(&words), Err(..));
    }

    #[test]
    #[should_panic]
    fn temperatures_index_out_of_bounds() {
        let words = [0; NUMBER_OF_CHANNELS];
        let temps = Temperatures::decode_from_holding_registers(&words).unwrap();
        let _ = temps[NUMBER_OF_CHANNELS]; // Index 8 is invalid
    }

    #[test]
    fn temperatures_display() {
        let words = [219, 65424, 32768, 0, 0, 0, 0, 0];
        let temps = Temperatures::decode_from_holding_registers(&words).unwrap();
        assert_eq!(
            temps.to_string(),
            "21.9, -11.2, NAN, 0.0, 0.0, 0.0, 0.0, 0.0"
        );
    }

    // --- TemperatureCorrection Tests ---
    #[test]
    fn temperature_correction_decode() {
        // Uses the same Temperature decoding logic
        let words = [10, 65526, 0, 32768, 1, 2, 3, 4]; // 1.0, -1.0, 0.0, NAN etc.
        let corrections = TemperatureCorrection::decode_from_holding_registers(&words).unwrap();

        assert_eq!(corrections[0], Temperature::try_from(1.0).unwrap());
        assert_eq!(corrections[1], Temperature::try_from(-1.0).unwrap());
        assert_eq!(corrections[2], Temperature::try_from(0.0).unwrap());
        assert!(corrections[3].is_nan());
        assert_eq!(corrections.as_slice().len(), 8);
    }

    #[test]
    fn temperature_correction_encode_single() {
        // Uses the same Temperature encoding logic
        assert_eq!(
            TemperatureCorrection::encode_for_write_register(Temperature::try_from(2.0).unwrap()),
            Ok(20)
        );
        assert_eq!(
            TemperatureCorrection::encode_for_write_register(Temperature::try_from(-1.5).unwrap()),
            Ok((-1.5 * 10.0) as i16 as Word)
        ); // 65521
        assert_eq!(
            TemperatureCorrection::encode_for_write_register(Temperature::try_from(0.0).unwrap()),
            Ok(0)
        );
    }

    #[test]
    fn temperature_correction_encode_nan() {
        // Encoding NAN might depend on desired behavior, assuming 0x8000 based on Temperature encode
        assert_matches!(
            TemperatureCorrection::encode_for_write_register(Temperature::NAN),
            Err(..)
        );
    }

    #[test]
    fn temperature_correction_channel_address() {
        assert_eq!(
            TemperatureCorrection::channel_address(Channel::try_from(0).unwrap()),
            TemperatureCorrection::ADDRESS + 0
        ); // 0x0008
        assert_eq!(
            TemperatureCorrection::channel_address(Channel::try_from(1).unwrap()),
            TemperatureCorrection::ADDRESS + 1
        ); // 0x0009
        assert_eq!(
            TemperatureCorrection::channel_address(Channel::try_from(7).unwrap()),
            TemperatureCorrection::ADDRESS + 7
        ); // 0x000F

        assert_eq!(TemperatureCorrection::ADDRESS, 0x0008);
        assert_eq!(TemperatureCorrection::QUANTITY, 8);
    }

    #[test]
    fn temperature_correction_display() {
        let words = [10, 65526, 0, 32768, 0, 0, 0, 0];
        let corr = TemperatureCorrection::decode_from_holding_registers(&words).unwrap();
        assert_eq!(corr.to_string(), "1.0, -1.0, 0.0, NAN, 0.0, 0.0, 0.0, 0.0");
    }

    // --- AutomaticReport Tests ---
    #[test]
    fn automatic_report_decode() {
        assert_matches!(AutomaticReport::decode_from_holding_registers(&[0x0000]).unwrap(), report if *report == 0);
        assert_matches!(AutomaticReport::decode_from_holding_registers(&[0x0001]).unwrap(), report if *report == 1);
        assert_matches!(AutomaticReport::decode_from_holding_registers(&[0x000A]).unwrap(), report if *report == 10); // 10 seconds
        assert_matches!(AutomaticReport::decode_from_holding_registers(&[0x00FF]).unwrap(), report if *report == 255); // Max seconds
    }

    #[test]
    fn automatic_report_decode_invalid() {
        assert_matches!(
            AutomaticReport::decode_from_holding_registers(&[0x0100]),
            Err(..)
        ); // Value 256 is invalid
    }

    #[test]
    fn automatic_report_encode() {
        assert_eq!(AutomaticReport(0).encode_for_write_register(), 0);
        assert_eq!(AutomaticReport(1).encode_for_write_register(), 1);
        assert_eq!(AutomaticReport(10).encode_for_write_register(), 10);
        assert_eq!(AutomaticReport(255).encode_for_write_register(), 255);
        assert_eq!(AutomaticReport::DISABLED.encode_for_write_register(), 0);
    }

    #[test]
    fn automatic_report_try_from_duration() {
        assert_matches!(AutomaticReport::try_from(Duration::ZERO), Ok(r) if r.as_secs() == 0);
        assert_matches!(AutomaticReport::try_from(Duration::from_secs(1)), Ok(r) if r.as_secs() == 1);
        assert_matches!(AutomaticReport::try_from(Duration::from_secs(10)), Ok(r) if r.as_secs() == 10);
        assert_matches!(AutomaticReport::try_from(Duration::from_secs(255)), Ok(r) if r.as_secs() == 255);
        // Includes fractional part (ignored)
        assert_matches!(AutomaticReport::try_from(Duration::from_millis(10500)), Ok(r) if r.as_secs() == 10);

        // Invalid duration
        let invalid_secs = (AutomaticReport::DURATION_MAX as u64) + 1;
        assert_matches!(AutomaticReport::try_from(Duration::from_secs(invalid_secs)), Err(ErrorDurationOutOfRange(value_secs)) if value_secs == invalid_secs);
    }

    #[test]
    fn automatic_report_helpers() {
        let report_10s = AutomaticReport::try_from(10).unwrap();
        let report_disabled = AutomaticReport::DISABLED;

        assert_eq!(report_10s.as_secs(), 10);
        assert_eq!(report_10s.as_duration(), Duration::from_secs(10));
        assert!(!report_10s.is_disabled());

        assert_eq!(report_disabled.as_secs(), 0);
        assert_eq!(report_disabled.as_duration(), Duration::ZERO);
        assert!(report_disabled.is_disabled());
    }

    #[test]
    fn automatic_report_display() {
        assert_eq!(AutomaticReport(0).to_string(), "0s");
        assert_eq!(AutomaticReport(1).to_string(), "1s");
        assert_eq!(AutomaticReport(255).to_string(), "255s");
    }

    // --- FactoryReset Tests ---
    #[test]
    fn factory_reset_encode() {
        assert_eq!(FactoryReset::encode_for_write_register(), 5);
    }

    #[test]
    fn factory_reset_address() {
        // Ensure it uses the same address as BaudRate write
        assert_eq!(FactoryReset::ADDRESS, BaudRate::ADDRESS);
    }
}
