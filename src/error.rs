use std::fmt;

#[derive(Debug)]
pub enum Error {
    RangeError,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::RangeError => write!(f, "Value out of range"),
        }
    }
}
