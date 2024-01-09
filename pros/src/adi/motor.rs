use crate::adi::{
    AdiError,
    port::AdiPort
};

use pros_sys::PROS_ERR;

use core::ops::{
    Deref,
    DerefMut
};

use crate::error::bail_on;

pub struct AdiMotor<'a> {
    port: &'a AdiPort,
}

impl<'a> AdiMotor<'a> {
    pub fn new(port: &mut AdiPort) -> Self {
        Self { port }
    }

    pub fn set_value(&mut self, value: i8) -> Result<(), AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_set(*self.port.deref(), value)) })
    }

    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_get(*self.port.deref())) })
    }

    pub fn stop(&mut self) -> Result<(), AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_motor_stop(*self.port.deref())) })
    }
}

impl<'a> Deref for AdiMotor<'a> {
    type Target = AdiPort;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl<'a> DerefMut for AdiMotor<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}