use crate::adi::{
    AdiError,
    port::AdiPort
};

use core::ffi::c_double;

use pros_sys::PROS_ERR;

use core::ops::{
    Deref,
    DerefMut
};

use crate::error::bail_on;

pub struct AdiGyro<'a> {
    port: &'a AdiPort,
    reference: i32
}

impl<'a> AdiGyro<'a> {
    pub unsafe fn new(port: &mut AdiPort, multiplier: c_double) -> Self {
        Self {
            port: port,
            reference: pros_sys::adi_gyro_init(**port.deref(), multiplier)
        }
    }

    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_gyro_get(self.reference)) })
    }

    pub fn reset(&mut self) -> Result<(), AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_gyro_reset(self.reference)) })
    }
}

impl<'a> Deref for AdiGyro<'a> {
    type Target = AdiPort;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl<'a> DerefMut for AdiGyro<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}