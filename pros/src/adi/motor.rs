use crate::adi::{
    AdiError,
    AdiSlot,
    New
};

use pros_sys::PROS_ERR;

use core::panic;

use crate::error::bail_on;

pub struct AdiMotor {
    port: u8
}

impl AdiMotor {
    /// Create an AdiMotor without checking if it is valid.
    ///
    /// # Safety
    ///
    /// The port must be above 0 and below [`pros_sys::NUM_ADI_PORTS`].
    pub unsafe fn new_unchecked(port: AdiSlot) -> Self {
        Self {
            port: port as u8
        }
    }

    /// Create an AdiMotor, panicking if the port is invalid.
    pub fn new_raw(slot: AdiSlot) -> Self {
        let port = slot as u8;
        if port < 1 || port > {pros_sys::NUM_ADI_PORTS as u8} {
            panic!("Invalid ADI port");
        }
        Self { port }
    }

    /// Create an AdiMotor, returning err `AdiError::InvalidPort` if the port is invalid.
    pub fn new(slot: AdiSlot) -> Result<Self, AdiError> {
        let port = slot as u8;
        if port < 1 || port > {pros_sys::NUM_ADI_PORTS as u8} {
            return Err(AdiError::InvalidPort);
        }
        Ok(Self { port })
    }

    /// Sets the speed of the given motor.
    pub fn set_value(&self, value: i8) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_set(self.port, value)) })
    }

    /// Returns the last set speed of the motor on the given port.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_get(self.port)) })
    }

    /// Stops the given motor.
    pub fn stop(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_stop(self.port)) })
    }
}

impl New for AdiMotor {
    fn new(slot: AdiSlot) -> Result<Self, AdiError> {
        Self::new(slot)
    }

    fn new_raw(slot: AdiSlot) -> Self {
        Self::new_raw(slot)
    }

    unsafe fn new_unchecked(slot: AdiSlot) -> Self {
        Self::new_unchecked(slot)
    }
}