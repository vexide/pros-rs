#![no_std]
#![no_main]

use pros::prelude::*;
use pros::adi::{
    AdiSlot,
    motor::AdiMotor,
    ultrasonic::AdiUltrasonic
};

#[derive(Default)]
pub struct Robot;
#[async_trait]
impl AsyncRobot for Robot {
    async unsafe fn opcontrol(&mut self) -> pros::Result {
        let mut pot = AdiPotentiometer::new(1);
        let mut ultrasonic = AdiUltrasonic::new((AdiSlot::A, AdiSlot::B));
        let angle = pot.angle()?;
        Ok(())
    }
}
async_robot!(Robot);
