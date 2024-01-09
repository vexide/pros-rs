use crate::adi::AdiError;

use core::ffi::c_double;

use pros_sys::PROS_ERR;

use core::ops::{
    Deref,
    DerefMut
};

use crate::error::bail_on;

pub struct AdiGyro {
    port: u8,
    reference: i32
}

impl AdiGyro {
    pub unsafe fn new(port: u8, multiplier: c_double) -> Self {
        Self {
            port,
            reference: pros_sys::adi_gyro_init(port, multiplier)
        }
    }

    pub fn value(&self) -> Result<f64, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_gyro_get(self.reference)) })
    }

    pub fn reset(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_gyro_reset(self.reference)) })
    }
}

impl Deref for AdiGyro {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl DerefMut for AdiGyro {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}