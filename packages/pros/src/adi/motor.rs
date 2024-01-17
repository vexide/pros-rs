use core::panic;

use pros_sys::PROS_ERR;

use crate::{
    adi::{AdiError, AdiSlot},
    error::bail_on,
};

pub struct AdiMotor {
    port: u8,
}

impl AdiMotor {
    /// Create an AdiMotor, returning err `AdiError::InvalidPort` if the port is invalid.
    pub fn new(slot: AdiSlot) -> Result<Self, AdiError> {
        let port = slot as u8;
        unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_get(port)) };
        Ok(Self { port })
    }

    /// Sets the speed of the given motor.
    pub fn set_value(&mut self, value: i8) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_set(self.port, value)) })
    }

    /// Returns the last set speed of the motor on the given port.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_get(self.port)) })
    }

    /// Stops the given motor.
    pub fn stop(&mut self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_stop(self.port)) })
    }
}