pub const PARITY: &tokio_serial::Parity = &tokio_serial::Parity::None;
pub const STOP_BITS: &tokio_serial::StopBits = &tokio_serial::StopBits::One;
pub const DATA_BITS: &tokio_serial::DataBits = &tokio_serial::DataBits::Eight;

pub fn serial_port_builder(device: &String, baud_rate: u32) -> tokio_serial::SerialPortBuilder {
    tokio_serial::new(device, baud_rate)
        .parity(*PARITY)
        .stop_bits(*STOP_BITS)
        .data_bits(*DATA_BITS)
        .flow_control(tokio_serial::FlowControl::None)
}
