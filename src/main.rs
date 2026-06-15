use eve::sts3215::*;

fn _teleop() {
    let mut servo_follow = Sts3215::new("/dev/ttyACM0", 1_000_000, 5000).expect("fuh");
    let mut servo_teach = Sts3215::new("/dev/ttyACM1", 1_000_000, 5000).expect("fuh");
    loop {
        for id in 1..7 {
            let packet_teach = servo_teach.read(id, Sts3215MemoryTableRegister::CurrentLocation);

            let current_pos = match packet_teach.expect("NO") {
                Sts3215ResponsePacket::ReadResponse {
                    id: _,
                    working_condition: _,
                    data,
                } => data,
                _ => panic!("not found"),
            };
            let _packet = servo_follow.write(
                id,
                Sts3215MemoryTableRegister::TargetLocation,
                current_pos.to_le_bytes().to_vec(),
            );
        }
        //let packet = servo.write(0x01, Sts3215MemoryTableRegister::TARGET_LOCATION, vec![0x00, 0x08, 0x00, 0x00, 0xe8, 0x03]);
    }
}

fn main() {
    let mut servo = Sts3215::new("/dev/ttyACM0", 1_000_000, 5000).expect("fuh");
    loop {
        if let Ok(packet) = servo.read(4, Sts3215MemoryTableRegister::CurrentLocation)
            && let Sts3215ResponsePacket::ReadResponse {
                id: _,
                working_condition,
                data,
            } = packet
        {
            println!("{working_condition} {data}");
        }
    }
}
