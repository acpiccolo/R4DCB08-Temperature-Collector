//! This module provides common data structures and error types for the `tokio`
//! based clients.
//!
//! It defines the `Error` enum, which encapsulates all possible communication errors.
use crate::protocol as proto;

/// Represents all possible errors that can occur during Modbus communication.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Wraps `proto::Error`.
    #[error(transparent)]
    ProtocolError(#[from] proto::Error),

    /// Wraps `tokio_modbus::ExceptionCode`.
    #[error(transparent)]
    TokioExceptionError(#[from] tokio_modbus::ExceptionCode),

    /// Wraps `tokio_modbus::Error`.
    #[error(transparent)]
    TokioError(#[from] tokio_modbus::Error),
}

/// The result type for tokio operations.
pub(crate) type Result<T> = std::result::Result<T, crate::tokio_common::Error>;

/// The parity used for serial communication.
pub const PARITY: &tokio_serial::Parity = &tokio_serial::Parity::None;
/// The number of stop bits used for serial communication.
pub const STOP_BITS: &tokio_serial::StopBits = &tokio_serial::StopBits::One;
/// The number of data bits used for serial communication.
pub const DATA_BITS: &tokio_serial::DataBits = &tokio_serial::DataBits::Eight;

/// Creates a `tokio_serial::SerialPortBuilder` with the specified settings.
///
/// # Arguments
///
/// * `device` - The path to the serial port device (e.g., `/dev/ttyUSB0`).
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
