#![no_std]
#![no_main]

use pros::prelude::*;
use pros::adi::*;

#[derive(Default)]
pub struct Robot;
#[async_trait]
impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        let mut pot = AdiPotentiometer::new(1);
        let mut ultrasonic = AdiUltrasonic::new((2, 3));
        let angle = pot.angle()?;
        Ok(())
    }
}
async_robot!(Robot);
