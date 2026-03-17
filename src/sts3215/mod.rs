use serialport::TTYPort;
use std::io::{Write, Read};

pub struct Sts3215 {
    port: TTYPort,
}

enum Sts3215MemoryTableRegister {
    ID = 0x5,
    BAUDRATE = 0x6,
}

#[derive(Debug)]
pub struct PingReplyPacket {
    id: u8,
    working_condition: u8,
}

impl Sts3215 {
    // defualt baud rate for this servo should be 1_000_000
    pub fn new(port_name: &str, baud_rate: u32, timeout: u64) -> Result<Self, serialport::Error> {
        let port_build = serialport::new(port_name, baud_rate)
            .timeout(std::time::Duration::from_millis(timeout));
        let port = TTYPort::open(&port_build)?;
        Ok(Sts3215 { port })
    }

    pub fn ping(&mut self, id: u8) -> Result<PingReplyPacket, serialport::Error> {
        let check_sum: u8 = !(id.wrapping_add(0x02).wrapping_add(0x01));
        let bytes: [u8; 6] = [0xff, 0xff, id, 0x02, 01, check_sum];
        self.port.write(&bytes)?;

        let mut buffer = [0; 6];
        self.port.read(&mut buffer)?;
        
        Ok(PingReplyPacket {
            id: buffer[2],
            working_condition: buffer[4],
        })
    }
}
