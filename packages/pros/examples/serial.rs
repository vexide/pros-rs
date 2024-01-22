#![no_std]
#![no_main]

use core::time::Duration;

use pros::prelude::*;

pub struct Robot {
    serial_port: SerialPort,
}

impl Robot {
    pub fn new(peripherals: Peripherals) -> Self {
        Self {
            serial_port: SerialPort::open(peripherals.port_1, 9600).expect("Failed to open port"),
        }
    }
}

impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        let mut buffer = [0; 256];

        loop {
            let read = self.serial_port.read(&mut buffer).unwrap();
            self.serial_port.write(&buffer[..read]).unwrap();

            pros::task::delay(Duration::from_millis(10));
        }
    }
}

async_robot!(Robot, Robot::new(Peripherals::take().unwrap()));
