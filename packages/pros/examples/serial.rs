#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
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
        let mut buffer = Vec::new();

        loop {
            self.serial_port.read(&mut buffer).unwrap();

            pros::task::delay(Duration::from_millis(10));
        }
    }
}

async_robot!(Robot, Robot::new(Peripherals::take().unwrap()));
