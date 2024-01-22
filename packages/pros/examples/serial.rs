#![no_std]
#![no_main]

use pros::prelude::*;

#[derive(Default)]
pub struct Robot {
    peripherals: Peripherals,
}

impl Robot {
    pub fn new(peripherals: Peripherals) {
        Self { peripherals }
    }
}

impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        let port = SerialPort::open(self.peripherals.port_1, 9600).expect("Failed to open port");

        let mut buffer = [0; 256];
        loop {
            let read = port.read(&mut buffer)?;
            port.write(&buffer[..read])?;
        }

        Ok(())
    }
}

async_robot!(Robot, Robot::new(Peripherals::take().unwrap()));
