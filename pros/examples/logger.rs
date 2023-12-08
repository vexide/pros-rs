#![no_std]
#![no_main]

use core::time::Duration;
use pros::prelude::*;
use pros_sys::delay;

#[derive(Debug, Default)]
struct ExampleRobot;
impl Robot for ExampleRobot {
    fn opcontrol(&mut self) -> pros::Result {
        pros::logger::ProsLogger::init().unwrap();

        let str = "PROS";
        loop {
            info!("Hello, {str}!");

            sleep(Duration::from_millis(1000));
        }

        Ok(())
    }
}
robot!(ExampleRobot);
