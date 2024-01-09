use crate::adi::{
    AdiError,
    AdiSlot
};

use pros_sys::PROS_ERR;

use core::panic;

use crate::error::bail_on;

pub struct AdiMotor {
    port: u8
}

impl AdiMotor {
    pub fn new(slot: AdiSlot) -> Self {
        let port = slot as u8;
        Self { port }
    }

    pub fn set_value(&self, value: i8) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_set(self.port, value)) })
    }

    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_get(self.port)) })
    }

    pub fn stop(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_stop(self.port)) })
    }
}