#![no_std]
#![no_main]

use pros::prelude::*;

#[derive(Default)]
pub struct Robot;

impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        println!("basic example");

        Motor::new(25, BrakeMode::None).unwrap();

        Ok(())
    }
}
async_robot!(Robot);
