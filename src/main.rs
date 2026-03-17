extern crate serialport;
use serialport::TTYPort;
use std::io::{Write, Read};

fn main() {
    let port_build = serialport::new("/dev/ttyACM0", 1000000).timeout(std::time::Duration::from_millis(5000));
    let mut tty_port = TTYPort::open(&port_build).expect("Failed to open port");
    println!("Port opened successfully!");
    let bytes: [u8; 8] = [0xff, 0xff, 0x01, 0x04, 0x02, 0x38, 0x02, 0xBE];
    let bytes = tty_port.write(&bytes).expect("Failed to write to port");
    tty_port.flush().expect("Failed to flush port");
    println!("Bytes written: {}", bytes);

    let mut buffer = [0; 64];
    tty_port.read(&mut buffer).expect("Failed to read from port");
    

    println!("Bytes read: {:?}", buffer);

}
