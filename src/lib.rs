mod error;
mod imp;

pub use error::Error;

pub use imp::{
    BaudRate, FACTORY_DEFAULT_ADDRESS, FACTORY_DEFAULT_BAUD_RATE, READ_ADDRESS_BROADCAST_ADDRESS,
};

#[cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync"))]
pub mod tokio_sync_client;

#[cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))]
pub mod tokio_async_client;

#[cfg(any(feature = "tokio-rtu", feature = "tokio-rtu-sync"))]
pub mod tokio_serial;
