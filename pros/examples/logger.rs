#![no_std]
#![no_main]

use core::time::Duration;
use pros::prelude::*;

#[derive(Debug, Default)]
struct ExampleRobot;
impl Robot for ExampleRobot {
    fn opcontrol(&mut self) -> pros::Result {
        pros::logger::ProsLogger::init().unwrap();

        let str = "PROS";
        info!("Hello, {str}!");

        Ok(())
    }
}
robot!(ExampleRobot);
