pub mod sts3215;
use sts3215::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_test() {
        let mut servo =
            sts3215::Sts3215::new("/dev/ttyACM0", 1_000_000, 5000).expect("Failed to create servo");
        for i in 1..6 {
            println!("id: {:?}", servo.read(i, Sts3215MemoryTableRegister::Id));
        }

        loop {
            println!(
                "current pos: {:?}",
                servo.read(3, Sts3215MemoryTableRegister::CurrentLocation)
            );
        }
    }
}
