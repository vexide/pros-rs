use crate::adi::AdiError;

use pros_sys::{
    PROS_ERR,
    adi_encoder_t
};

use core::ops::{
    Deref,
    DerefMut
};

use crate::error::bail_on;

pub struct AdiEncoder {
    port_top: u8,
    port_bottom: u8,
    reverse: bool,
    reference: adi_encoder_t
}

impl AdiEncoder {
    pub unsafe fn new(port_top: u8, port_bottom: u8, reverse: bool) -> Self {
        Self {
            port_top,
            port_bottom,
            reverse,
            reference: pros_sys::adi_encoder_init(port_top, port_bottom, reverse)
        }
    }

    pub fn reset(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_encoder_reset(self.reference)) })
    }

    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_encoder_get(self.reference)) })
    }
}

impl Deref for AdiEncoder {
    type Target = (u8, u8);
    fn deref(&self) -> &Self::Target {
        &(self.port_top, self.port_bottom)
    }
}

impl DerefMut for AdiEncoder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut (self.port_top, self.port_bottom)
    }
}