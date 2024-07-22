use crate::protocol;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "The channel value {0} is outside the permissible range of {} to {}",
        protocol::CHANNELS_MIN,
        protocol::CHANNELS_MAX
    )]
    ChannelOutOfRange(u8),
    #[error(
        "The degree celsius value {0} is outside the permissible range of {} to {}",
        protocol::DEGREE_CELSIUS_MIN,
        protocol::DEGREE_CELSIUS_MAX
    )]
    DegreeCelsiusOutOfRange(f32),
    #[error(
        "The duration value {0} is outside the permissible range of {} to {}",
        protocol::DURATION_MIN,
        protocol::DURATION_MAX
    )]
    DurationOutOfRange(u8),
    #[error(
        "The address value {0} is outside the permissible range of {} to {}",
        protocol::ADDRESS_MIN,
        protocol::ADDRESS_MAX
    )]
    AddressOutOfRange(u8),
}
