use crate::adi::{
    AdiError,
    AdiSlot
};

use pros_sys::PROS_ERR;

use crate::error::bail_on;

pub struct AdiPotentiometer {
    port: u8,
    reference: i32
}

impl AdiPotentiometer {
    pub unsafe fn new(port: AdiSlot) -> Self {
        Self {
            port: port as u8,
            reference: pros_sys::adi_potentiometer_init(port as u8)
        }
    }

    pub fn angle(&self) -> Result<f64, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_potentiometer_get_angle(self.reference)) })
    }
}