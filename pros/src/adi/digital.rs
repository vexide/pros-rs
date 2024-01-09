
use core::ops::{Deref, DerefMut};

use crate::error::bail_on;

use pros_sys::{PROS_ERR, E_ADI_DIGITAL_IN, E_ADI_DIGITAL_OUT };

use crate::adi::{AdiError, port::AdiPort};

pub struct AdiDigitalIn<'a> {
    port: &'a AdiPort,
}

impl<'a> AdiDigitalIn<'a> {
    pub fn new(port: &mut AdiPort) -> Self {
        port.set_config(E_ADI_DIGITAL_IN).unwrap();
        Self { port }
    }

    pub fn new_press(&self) -> Result<bool, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_get_new_press(*self.port.deref())) != 0 })
    }

    pub fn value(&self) -> Result<bool, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_read(*self.port.deref())) != 0 })
    }
}

impl<'a> Deref for AdiDigitalIn<'a> {
    type Target = AdiPort;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl<'a> DerefMut for AdiDigitalIn<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}

pub struct AdiDigitalOut<'a> {
    port: &'a AdiPort,
}

impl<'a> AdiDigitalOut<'a> {
    pub fn new(port: &mut AdiPort) -> Self {
        port.set_config(E_ADI_DIGITAL_OUT).unwrap();
        Self { port }
    }

    pub fn set_value(&mut self, value: bool) -> Result<(), AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_write(*self.port.deref(), value)) })
    }
}

impl<'a> Deref for AdiDigitalOut<'a> {
    type Target = AdiPort;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl<'a> DerefMut for AdiDigitalOut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}