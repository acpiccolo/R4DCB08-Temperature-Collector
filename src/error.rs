use crate::protocol;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "The channel value {0} is outside the permissible range of {min} to {max}",
        min = protocol::CHANNELS_MIN,
        max = protocol::CHANNELS_MAX
    )]
    ChannelOutOfRange(u8),
    #[error(
        "The degree celsius value {0} is outside the permissible range of {min} to {max}",
        min = protocol::DEGREE_CELSIUS_MIN,
        max = protocol::DEGREE_CELSIUS_MAX
    )]
    DegreeCelsiusOutOfRange(f32),
    #[error(
        "The duration value {0} is outside the permissible range of {min} to {max}",
        min = protocol::DURATION_MIN,
        max = protocol::DURATION_MAX
    )]
    DurationOutOfRange(u8),
    #[error(
        "The address value {0} is outside the permissible range of {min} to {max}",
        min = protocol::ADDRESS_MIN,
        max = protocol::ADDRESS_MAX
    )]
    AddressOutOfRange(u8),
}
