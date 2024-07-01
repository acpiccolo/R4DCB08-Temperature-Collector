use std::fmt;

#[derive(Debug)]
pub enum Error {
    RangeError,
    Error(tokio_modbus::Error),
    Exception(tokio_modbus::Exception),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            Error::Error(ref err) => write!(f, "Modbus error: {}", err),
            Error::Exception(ref err) => write!(f, "Modbus exception: {}", err),
            Error::RangeError => write!(f, "Value out of range"),
        }
    }
}

impl From<tokio_modbus::Error> for Error {
    fn from(err: tokio_modbus::Error) -> Error {
        Error::Error(err)
    }
}

impl From<tokio_modbus::Exception> for Error {
    fn from(err: tokio_modbus::Exception) -> Error {
        Error::Exception(err)
    }
}
