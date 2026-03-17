use serialport::TTYPort;
use std::io::{Read, Write};

pub struct Sts3215 {
    port: TTYPort,
}

pub enum Sts3215MemoryTableRegister {
    ID,
    BAUDRATE,
    CURRENT_LOCATION,
    OPERATION_MODE,
    TARGET_LOCATION,
}

impl Sts3215MemoryTableRegister {
    fn address(&self) -> u8 {
        match self {
            Sts3215MemoryTableRegister::ID => 0x5,
            Sts3215MemoryTableRegister::BAUDRATE => 0x6,
            Sts3215MemoryTableRegister::OPERATION_MODE => 0x21,
            Sts3215MemoryTableRegister::CURRENT_LOCATION => 0x38,
            Sts3215MemoryTableRegister::TARGET_LOCATION => 0x2A,
        }
    }

    fn size(&self) -> u8 {
        match self {
            Sts3215MemoryTableRegister::ID => 1,
            Sts3215MemoryTableRegister::BAUDRATE => 1,
            Sts3215MemoryTableRegister::OPERATION_MODE => 1,
            Sts3215MemoryTableRegister::CURRENT_LOCATION => 2,
            Sts3215MemoryTableRegister::TARGET_LOCATION => 2,
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
    pub fn new(port_name: &str, baud_rate: u32, timeout: u64) -> Result<Self, serialport::Error> {
        let port_build = serialport::new(port_name, baud_rate)
            .timeout(std::time::Duration::from_millis(timeout));
        let port = TTYPort::open(&port_build)?;
        Ok(Sts3215 { port })
    }

    pub fn ping(&mut self, id: u8) -> Result<Sts3215ResponsePacket, serialport::Error> {
        let check_sum: u8 = !(id.wrapping_add(0x02).wrapping_add(0x01));
        let bytes: [u8; 6] = [0xff, 0xff, id, 0x02, 01, check_sum];
        self.port.write(&bytes)?;

        let mut buffer = [0; 6];
        self.port.read(&mut buffer)?;

        Ok(Sts3215ResponsePacket::PingResponse {
            id: buffer[2],
            working_condition: buffer[4],
        })
    }

    pub fn read(
        &mut self,
        id: u8,
        register: Sts3215MemoryTableRegister,
    ) -> Result<Sts3215ResponsePacket, serialport::Error> {
        let check_sum: u8 = !(id
            .wrapping_add(0x4) // effective data length
            .wrapping_add(0x2) // read instruction ID
            .wrapping_add(register.address())
            .wrapping_add(register.size()));
        let bytes = [
            0xff,
            0xff,
            id,
            0x4,
            0x2,
            register.address(),
            register.size(),
            check_sum,
        ];
        self.port.write(&bytes)?;

        let mut buffer: [u8; 64] = [0; 64];
        self.port.read(&mut buffer)?;

        // todo: assert that length matches what we expect
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
    ) -> Result<Sts3215ResponsePacket, serialport::Error> {
        let mut bytes = vec![
            0xff,
            0xff,
            id,
            data.len() as u8 + 3,
            0x3,
            register.address(),
        ];
        for byte in data {
            bytes.push(byte);
        }

        let mut check_sum = 0_u8;
        for byte in &bytes[2..] {
            check_sum = check_sum.wrapping_add(*byte);
        }
        bytes.push(!check_sum);
        println!("bytes {:x?}", bytes);

        self.port.write(&bytes as &[u8])?;

        let mut buffer: [u8; 64] = [0; 64];
        self.port.read(&mut buffer)?;
        Ok(Sts3215ResponsePacket::WriteResponse {
            id: buffer[2],
            working_condition: buffer[4],
        })
    }
}
