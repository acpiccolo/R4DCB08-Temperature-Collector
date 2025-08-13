//! This module provides common data structures and error types for the `tokio`
//! based clients.
//!
//! It defines the `Error` enum, which encapsulates all possible communication errors.

use crate::protocol as proto;

/// Represents all possible errors that can occur during Modbus communication.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error originating from the protocol logic, such as invalid data.
    #[error(transparent)]
    Protocol(#[from] proto::Error),

    /// A Modbus exception response from the device (e.g., "Illegal Function").
    #[error(transparent)]
    ModbusException(#[from] tokio_modbus::ExceptionCode),

    /// A transport or communication error from the underlying `tokio-modbus` client.
    #[error(transparent)]
    Modbus(#[from] tokio_modbus::Error),
}

/// The result type for tokio operations.
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// The parity used for serial communication.
pub const PARITY: &tokio_serial::Parity = &tokio_serial::Parity::None;
/// The number of stop bits used for serial communication.
pub const STOP_BITS: &tokio_serial::StopBits = &tokio_serial::StopBits::One;
/// The number of data bits used for serial communication.
pub const DATA_BITS: &tokio_serial::DataBits = &tokio_serial::DataBits::Eight;

/// Creates and configures a `tokio_serial::SerialPortBuilder` for RTU communication.
///
/// This function sets up the standard communication parameters required by the
/// R4DCB08 device: no parity, 8 data bits, and 1 stop bit.
///
/// Note that this function only creates and configures the builder. It does not
/// open the serial port, and therefore does not perform any I/O and cannot fail.
/// The actual connection is established when this builder is used by a `tokio-modbus`
/// client constructor.
///
/// # Arguments
///
/// * `device` - The path to the serial port device (e.g., `/dev/ttyUSB0` on Linux
///   or `COM3` on Windows).
/// * `baud_rate` - The baud rate for the serial communication.
pub fn serial_port_builder(
    device: &str,
    baud_rate: &proto::BaudRate,
) -> tokio_serial::SerialPortBuilder {
    tokio_serial::new(device, *baud_rate as u32)
        .parity(*PARITY)
        .stop_bits(*STOP_BITS)
        .data_bits(*DATA_BITS)
        .flow_control(tokio_serial::FlowControl::None)
}
