
use core::ops::{Deref, DerefMut};

use crate::error::bail_on;

use pros_sys::PROS_ERR;

use crate::adi::AdiError;

pub struct AdiDigitalIn {
    port: u8,
}

impl AdiDigitalIn {
    pub fn new(port: u8) -> Self {
        Self { port }
    }

    pub fn new_press(&self) -> Result<bool, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_get_new_press(self.port)) != 0 })
    }

    pub fn value(&self) -> Result<bool, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_read(self.port)) != 0 })
    }
}

impl Deref for AdiDigitalIn {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl DerefMut for AdiDigitalIn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}

pub struct AdiDigitalOut {
    port: u8,
}

impl AdiDigitalOut {
    pub fn new(port: u8) -> Self {
        Self { port }
    }

    pub fn set_value(&mut self, value: bool) -> Result<(), AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_write(self.port, value)) })
    }
}

impl Deref for AdiDigitalOut {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl DerefMut for AdiDigitalOut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}