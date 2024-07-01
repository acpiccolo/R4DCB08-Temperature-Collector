use std::fmt;

#[derive(Debug)]
pub enum Error {
    R4DCB08Error(crate::Error),
    ModbusError(tokio_modbus::Error),
    ModbusException(tokio_modbus::Exception),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            Error::R4DCB08Error(ref err) => write!(f, "R4DCB08 error: {}", err),
            Error::ModbusError(ref err) => write!(f, "Modbus error: {}", err),
            Error::ModbusException(ref err) => write!(f, "Modbus exception: {}", err),
        }
    }
}

impl From<crate::Error> for Error {
    fn from(err: crate::Error) -> Error {
        Error::R4DCB08Error(err)
    }
}

impl From<tokio_modbus::Error> for Error {
    fn from(err: tokio_modbus::Error) -> Error {
        Error::ModbusError(err)
    }
}

impl From<tokio_modbus::Exception> for Error {
    fn from(err: tokio_modbus::Exception) -> Error {
        Error::ModbusException(err)
    }
}
