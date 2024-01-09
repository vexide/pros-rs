use crate::adi::{
    AdiError,
    AdiSlot
};

use core::ffi::c_double;

use pros_sys::PROS_ERR;

use crate::error::bail_on;

pub struct AdiGyro {
    port: u8,
    reference: i32
}

impl AdiGyro {
    pub unsafe fn new(port: AdiSlot, multiplier: c_double) -> Self {
        Self {
            port: port as u8,
            reference: pros_sys::adi_gyro_init(port as u8, multiplier)
        }
    }

    pub fn value(&self) -> Result<f64, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_gyro_get(self.reference)) })
    }

    pub fn reset(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_gyro_reset(self.reference)) })
    }
}