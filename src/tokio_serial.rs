pub const PARITY: &tokio_serial::Parity = &tokio_serial::Parity::None;
pub const STOP_BITS: &tokio_serial::StopBits = &tokio_serial::StopBits::One;
pub const DATA_BITS: &tokio_serial::DataBits = &tokio_serial::DataBits::Eight;
pub const BAUD_RATE: u32 = 9600;

pub fn serial_port_builder(device: &String) -> tokio_serial::SerialPortBuilder {
    tokio_serial::new(device, BAUD_RATE)
        .parity(*PARITY)
        .stop_bits(*STOP_BITS)
        .data_bits(*DATA_BITS)
        .flow_control(tokio_serial::FlowControl::None)
}
