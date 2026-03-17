mod sts3215;

pub fn test_ping() {
    let mut servo = sts3215::Sts3215::new("/dev/ttyACM0", 1_000_000, 5000).expect("Failed to create servo");
    let packet = servo.ping(0x01).expect("Failed to ping servo");
    println!("Ping reply: {:?}", packet);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

        test_ping()
    }
}
