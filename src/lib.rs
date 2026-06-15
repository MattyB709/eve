pub mod sts3215;

#[cfg(test)]
mod tests {
    use super::*;
    use sts3215::*;

    #[test]
    fn ping_test() {
        let mut servo = Sts3215::new("/dev/ttyACM0", 1_000_000, 5000).expect("couldn't open port");
        let packet = servo.ping(4);
        match packet {
            Ok(Sts3215ResponsePacket::PingResponse {
                id,
                working_condition,
            }) => {
                println!("id: {id} {working_condition}");
            }
            Err(servo_error) => {
                println!("error pinging servo: {:?}", servo_error);
            }
            _ => {
                println!("unexpected packet");
            }
        }
    }

    #[test]
    fn read_test() {
        let mut servo = Sts3215::new("/dev/ttyACM0", 1_000_000, 5000).expect("couldn't open port");
        loop {
            match servo.read(4, Sts3215MemoryTableRegister::CurrentLocation) {
                Ok(packet) => {
                    if let Sts3215ResponsePacket::ReadResponse {
                        id,
                        working_condition,
                        data,
                    } = packet
                    {
                        println!("id: {id} {working_condition} {data}");
                    }
                }
                Err(servo_error) => {
                    println!("error reading servo: {:?}", servo_error);
                    break;
                }
            }
        }
    }
}
