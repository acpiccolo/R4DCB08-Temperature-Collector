mod error;

pub use error::Error;
pub mod protocol;

#[cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync"))]
pub mod tokio_sync_client;

#[cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))]
pub mod tokio_async_client;

#[cfg(any(feature = "tokio-rtu", feature = "tokio-rtu-sync"))]
pub mod tokio_serial;
