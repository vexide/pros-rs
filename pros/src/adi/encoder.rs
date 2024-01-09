use crate::adi::{
    AdiError,
    port::AdiPort
};

use pros_sys::{
    PROS_ERR,
    adi_encoder_t
};

use core::ops::{
    Deref,
    DerefMut
};

use crate::error::bail_on;

pub struct AdiEncoder<'a> {
    port_top: &'a AdiPort,
    port_bottom: &'a AdiPort,
    reverse: bool,
    reference: adi_encoder_t
}

impl<'a> AdiEncoder<'a> {
    pub unsafe fn new(port_top: &mut AdiPort, port_bottom: &mut AdiPort, reverse: bool) -> Self {
        Self {
            port_top,
            port_bottom,
            reverse,
            reference: pros_sys::adi_encoder_init(**port_top.deref(), **port_bottom.deref(), reverse)
        }
    }

    pub fn reset(&mut self) -> Result<(), AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_encoder_reset(self.reference)) })
    }

    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_encoder_get(self.reference)) })
    }
}

impl<'a> Deref for AdiEncoder<'a> {
    type Target = (&'a AdiPort, &'a AdiPort);
    fn deref(&self) -> &Self::Target {
        &(self.port_top, self.port_bottom)
    }
}

impl<'a> DerefMut for AdiEncoder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut (self.port_top, self.port_bottom)
    }
}