use crate::adi::AdiError;

use pros_sys::PROS_ERR;

use core::{ops::{
    Deref,
    DerefMut
}, panic};

use crate::error::bail_on;

pub struct AdiMotor {
    port: u8
}

impl AdiMotor {
    pub fn new(port: u8) -> Self {
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

impl Deref for AdiMotor {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl DerefMut for AdiMotor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}