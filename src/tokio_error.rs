#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("R4DCB08 error: {0}")]
    R4DCB08Error(#[from] crate::Error),
    #[error("Modbus error: {0}")]
    ModbusError(#[from] tokio_modbus::Error),
    #[error("Modbus exception: {0}")]
    ModbusException(#[from] tokio_modbus::ExceptionCode),
}
