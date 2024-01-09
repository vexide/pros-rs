use crate::error::bail_on;

use pros_sys::PROS_ERR;

use crate::adi::{
    AdiError,
    AdiSlot
};

pub struct AdiDigitalIn {
    port: u8,
}

impl AdiDigitalIn {
    pub fn new(slot: AdiSlot) -> Self {
        let port = slot as u8;
        Self { port }
    }

    pub fn new_press(&self) -> Result<bool, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_get_new_press(self.port)) != 0 })
    }

    pub fn value(&self) -> Result<bool, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_read(self.port)) != 0 })
    }
}

pub struct AdiDigitalOut {
    port: u8,
}

impl AdiDigitalOut {
    pub fn new(slot: AdiSlot) -> Self {
        let port = slot as u8;
        Self { port }
    }

    pub fn set_value(&mut self, value: bool) -> Result<(), AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_write(self.port, value)) })
    }
}