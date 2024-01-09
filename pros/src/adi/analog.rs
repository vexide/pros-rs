use crate::adi::{
    AdiError,
    port::AdiPort
};

use crate::error::bail_on;

use pros_sys::{
    E_ADI_ANALOG_IN,
    E_ADI_ANALOG_OUT,
    PROS_ERR
};

use core::ops::{
    Deref,
    DerefMut
};

pub struct AdiAnalogIn<'a> {
    port: &'a AdiPort,
}

impl<'a> AdiAnalogIn<'a> {
    pub fn new(port: &mut AdiPort) -> Self {
        port.set_config(E_ADI_ANALOG_IN).unwrap();
        Self { port }
    }

    pub fn calibrate(&mut self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_analog_calibrate(*self.port.deref())) })
    }

    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_analog_read(*self.port.deref())) })
    }

    pub fn value_calibrated(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_analog_read_calibrated(*self.port.deref())) })
    }

    pub fn value_calibrated_hr(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_analog_read_calibrated_HR(*self.port.deref())) })
    }
}

impl<'a> Deref for AdiAnalogIn<'a> {
    type Target = AdiPort;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl<'a> DerefMut for AdiAnalogIn<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}

pub struct AdiAnalogOut<'a> {
    port: &'a AdiPort,
}

impl<'a> AdiAnalogOut<'a> {
    pub fn new(port: &mut AdiPort) -> Self {
        port.set_config(E_ADI_ANALOG_OUT).unwrap();
        Self { port }
    }

    pub fn set_value(&mut self, value: i32) -> Result<(), AdiError> {
        bail_on! {
            PROS_ERR,
            unsafe { pros_sys::adi_port_set_value(*self.port.deref(), value) }
        }
    }
}

impl<'a> Deref for AdiAnalogOut<'a> {
    type Target = AdiPort;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl<'a> DerefMut for AdiAnalogOut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}