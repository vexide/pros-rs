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

pub struct AdiPotentiometer<'a> {
    port: &'a AdiPort,
    reference: i32
}

impl<'a> AdiPotentiometer<'a> {
    pub unsafe fn new(port: &mut AdiPort) -> Self {
        Self {
            port: port,
            reference: pros_sys::adi_potentiometer_init(**port.deref())
        }
    }

    pub fn angle(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_potentiometer_get_angle(self.reference)) })
    }
}

impl Deref for AdiPotentiometer<'_> {
    type Target = AdiPort;
    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl DerefMut for AdiPotentiometer<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.port
    }
}
