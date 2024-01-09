use crate::adi::AdiError;

use pros_sys::PROS_ERR;

use core::ops::{
    Deref,
    DerefMut
};

use crate::error::bail_on;

pub struct AdiPotentiometer {
    port: u8,
    reference: i32
}

impl AdiPotentiometer {
    pub unsafe fn new(port: u8) -> Self {
        Self {
            port: port,
            reference: pros_sys::adi_potentiometer_init(port)
        }
    }

    pub fn angle(&self) -> Result<f64, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_potentiometer_get_angle(self.reference)) })
    }
}

impl Deref for AdiPotentiometer {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl DerefMut for AdiPotentiometer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}
