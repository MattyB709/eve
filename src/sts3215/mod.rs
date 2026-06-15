use bitflags::bitflags;
use serialport::TTYPort;
use std::io::{self, Read, Write};
use std::result::Result;

// This file follows the protocol outlined here: https://roar-qutrc.github.io/systems/st3215-protocol.html

pub struct Sts3215 {
    port: TTYPort,
}

#[derive(Debug)]
pub enum ServoError {
    Io(io::Error),
    InvalidResponse,
    ServoStatusError(StatusErrorFlags),
}

// these are the error flags that can be returned in the error byte of a response packet. This isn't documented super well in the English manual but the sdk
// uses these bits
bitflags! {
    #[derive(Debug)]
    pub struct StatusErrorFlags: u8 {
        const VOLTAGE_ERROR = 0b00000001;
        const ANGLE_LIMIT_ERROR = 0b00000010;
        const OVERHEATING_ERROR = 0b00000100;
        const OVERELECTRIC_ERROR = 0b00001000;
        const OVERLOAD_ERROR = 0b00010000;
    }
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

    // this function should be able to send any command to the servo and return the raw response buffer, plus handle error checking and checksum verification
    fn serial_send_and_recv(&mut self, buffer: Vec<u8>) -> Result<Vec<u8>, ServoError> {
        let mut formatted_buffer = vec![0xff, 0xff];
        let check_sum: u8 = !(buffer.iter().fold(0, |acc, x| acc.wrapping_add(*x)));
        formatted_buffer.extend(buffer);
        formatted_buffer.push(check_sum);
        self.port
            .write_all(formatted_buffer.as_slice())
            .map_err(ServoError::Io)?;

        let mut recv_buffer = vec![0; 4];

        self.port
            .read_exact(&mut recv_buffer)
            .map_err(ServoError::Io)?;

        // check header
        if recv_buffer[0] != 0xff || recv_buffer[1] != 0xff {
            return Err(ServoError::InvalidResponse);
        }

        let id = recv_buffer[2];
        let length = recv_buffer[3];
        let mut buffer_rest = vec![0; length as usize];
        self.port
            .read_exact(&mut buffer_rest)
            .map_err(ServoError::Io)?;

        let error = buffer_rest[0];

        if error != 0 {
            let servo_status_error = StatusErrorFlags::from_bits(error)
                .unwrap_or_else(|| panic!("Invalid error byte received from servo: {error}"));
            return Err(ServoError::ServoStatusError(servo_status_error));
        }

        // checksum is calculated using id, length, instruction, and parameters, with a negation
        let mut calculated_check_sum = buffer_rest[..buffer_rest.len() - 1]
            .iter()
            .fold(0u8, |acc, x| acc.wrapping_add(*x));
        calculated_check_sum = !calculated_check_sum.wrapping_add(id).wrapping_add(length);
        if calculated_check_sum != buffer_rest[buffer_rest.len() - 1] {
            return Err(ServoError::InvalidResponse);
        }

        recv_buffer.extend(buffer_rest);
        Ok(recv_buffer)
    }

    pub fn ping(&mut self, id: u8) -> Result<Sts3215ResponsePacket, ServoError> {
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
    ) -> Result<Sts3215ResponsePacket, ServoError> {
        let buffer =
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
    ) -> Result<Sts3215ResponsePacket, ServoError> {
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
