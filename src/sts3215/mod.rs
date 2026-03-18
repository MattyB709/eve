use serialport::TTYPort;
use std::io::{self, Read, Write};

pub struct Sts3215 {
    port: TTYPort,
}

pub enum Sts3215MemoryTableRegister {
    Id,
    Baudrate,
    CurrentLocation,
    OperationMode,
    TargetLocation,
}

impl Sts3215MemoryTableRegister {
    fn address(&self) -> u8 {
        match self {
            Sts3215MemoryTableRegister::Id => 0x5,
            Sts3215MemoryTableRegister::Baudrate => 0x6,
            Sts3215MemoryTableRegister::OperationMode => 0x21,
            Sts3215MemoryTableRegister::CurrentLocation => 0x38,
            Sts3215MemoryTableRegister::TargetLocation => 0x2A,
        }
    }

    fn size(&self) -> u8 {
        match self {
            Sts3215MemoryTableRegister::Id => 1,
            Sts3215MemoryTableRegister::Baudrate => 1,
            Sts3215MemoryTableRegister::OperationMode => 1,
            Sts3215MemoryTableRegister::CurrentLocation => 2,
            Sts3215MemoryTableRegister::TargetLocation => 2,
        }
    }
}

#[derive(Debug)]
pub enum Sts3215ResponsePacket {
    PingResponse {
        id: u8,
        working_condition: u8,
    },
    ReadResponse {
        id: u8,
        working_condition: u8,
        data: u16,
    },
    WriteResponse {
        id: u8,
        working_condition: u8,
    },
}

impl Sts3215 {
    // defualt baud rate for this servo should be 1_000_000
    pub fn new(port_name: &str, baud_rate: u32, timeout: u64) -> io::Result<Self> {
        let port_build = serialport::new(port_name, baud_rate)
            .timeout(std::time::Duration::from_millis(timeout));
        let port = TTYPort::open(&port_build)?;
        Ok(Sts3215 { port })
    }

    fn serial_send_and_recv(&mut self, buffer: Vec<u8>) -> io::Result<[u8; 64]> {
        let mut formatted_buffer = vec![0xff, 0xff];
        let check_sum: u8 = !(buffer.iter().fold(0, |acc, x| acc.wrapping_add(*x)));
        formatted_buffer.extend(buffer);
        formatted_buffer.push(check_sum);
        self.port.write(&mut formatted_buffer.as_slice())?;

        let mut recv_buffer: [u8; 64] = [0; 64];

        self.port.read(&mut recv_buffer)?;
        Ok(recv_buffer)
    }

    pub fn ping(&mut self, id: u8) -> io::Result<Sts3215ResponsePacket> {
        let buffer = self.serial_send_and_recv(vec![id, 0x02, 0x01])?;

        Ok(Sts3215ResponsePacket::PingResponse {
            id: buffer[2],
            working_condition: buffer[4],
        })
    }

    // todo: this should be able to read multiple registers at once and return a vector
    pub fn read(
        &mut self,
        id: u8,
        register: Sts3215MemoryTableRegister,
    ) -> io::Result<Sts3215ResponsePacket> {
        let buffer: [u8; 64] =
            self.serial_send_and_recv(vec![id, 0x4, 0x2, register.address(), register.size()])?;

        assert_eq!(buffer[3], register.size() + 2); // checking that effective data length is correct
        let output = if register.size() == 1 {
            buffer[5] as u16
        } else {
            u16::from_le_bytes(buffer[5..7].try_into().unwrap())
        };

        Ok(Sts3215ResponsePacket::ReadResponse {
            id: buffer[2],
            working_condition: buffer[4],
            data: output,
        })
    }

    pub fn write(
        &mut self,
        id: u8,
        register: Sts3215MemoryTableRegister,
        data: Vec<u8>,
    ) -> io::Result<Sts3215ResponsePacket> {
        let mut bytes = vec![id, data.len() as u8 + 3, 0x3, register.address()];
        for byte in data {
            bytes.push(byte);
        }
        let buffer = self.serial_send_and_recv(bytes)?;

        Ok(Sts3215ResponsePacket::WriteResponse {
            id: buffer[2],
            working_condition: buffer[4],
        })
    }
}
