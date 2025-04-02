/// Enum representing various errors that can occur while using the tokio_modbus library with the R4DCB08 temperature module.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error wrapping a tokio_modbus error.
    ///
    /// # Arguments
    ///
    /// * `0` - The underlying tokio_modbus error.
    #[error("Modbus error: {0}")]
    ModbusError(#[from] tokio_modbus::Error),
    /// Error wrapping a tokio_modbus exception code.
    ///
    /// # Arguments
    ///
    /// * `0` - The underlying tokio_modbus exception code.
    #[error("Modbus exception: {0}")]
    ModbusException(#[from] tokio_modbus::ExceptionCode),
}
