use crate::error::bail_on;

use pros_sys::PROS_ERR;

use crate::adi::{
    AdiError,
    AdiSlot,
    New
};

pub struct AdiDigitalIn {
    port: u8,
}

impl AdiDigitalIn {
    /// Create an AdiDigitalIn without checking if it is valid.
    /// 
    /// # Safety
    /// 
    /// The port must be above 0 and below [`pros_sys::NUM_ADI_PORTS`].
    pub  fn new_unchecked(slot: AdiSlot) -> Self {
        Self {
            port: slot as u8
        }
    }

    /// Create an AdiDigitalIn, panicking if the port is invalid.
    pub fn new(slot: AdiSlot) -> Self {
        let port = slot as u8;
        if port < 1 || port > {pros_sys::NUM_ADI_PORTS as u8} {
            panic!("Invalid ADI port");
        }
        Self { port }
    }

    /// Create an AdiDigitalIn, returning err `AdiError::InvalidPort` if the port is invalid.
    pub fn try_new(slot: AdiSlot) -> Result<Self, AdiError> {
        let port = slot as u8;
        if port < 1 || port > {pros_sys::NUM_ADI_PORTS as u8} {
            return Err(AdiError::InvalidPort);
        }
        Ok(Self { port })
    }

    /// Gets the current value of a digital input pin.
    pub fn new_press(&self) -> Result<bool, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_get_new_press(self.port)) != 0 })
    }

    /// Gets the current value of a digital input pin.
    pub fn value(&self) -> Result<bool, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_read(self.port)) != 0 })
    }
}

impl New for AdiDigitalIn {
    fn new(slot: AdiSlot) -> Result<Self, AdiError> {
        Self::try_new(slot)
    }

    fn new_raw(slot: AdiSlot) -> Self {
        Self::new(slot)
    }

    fn new_unchecked(slot: AdiSlot) -> Self {
        Self::new_unchecked(slot)
    }
}

pub struct AdiDigitalOut {
    port: u8,
}

impl AdiDigitalOut {
    /// Create an AdiDigitalOut without checking if it is valid.
    ///
    /// # Safety
    ///
    /// The port must be above 0 and below [`pros_sys::NUM_ADI_PORTS`].
    pub fn new_unchecked(slot: AdiSlot) -> Self {
        Self {
            port: slot as u8
        }
    }

    /// Create an AdiDigitalOut, panicking if the port is invalid.
    pub fn new_raw(slot: AdiSlot) -> Self {
        let port = slot as u8;
        if port < 1 || port > {pros_sys::NUM_ADI_PORTS as u8} {
            panic!("Invalid ADI port");
        }
        Self { port }
    }

    /// Create an AdiDigitalOut, returning err `AdiError::InvalidPort` if the port is invalid.
    pub fn new(slot: AdiSlot) -> Result<Self, AdiError> {
        let port = slot as u8;
        if port < 1 || port > {pros_sys::NUM_ADI_PORTS as u8} {
            return Err(AdiError::InvalidPort);
        }
        Ok(Self { port })
    }

    /// Sets the digital value (1 or 0) of a pin.
    pub fn set_value(&mut self, value: bool) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_digital_write(self.port, value)) })
    }
}

impl New for AdiDigitalOut {
    fn new(slot: AdiSlot) -> Result<Self, AdiError> {
        Self::new(slot)
    }

    fn new_raw(slot: AdiSlot) -> Self {
        Self::new_raw(slot)
    }

    fn new_unchecked(slot: AdiSlot) -> Self {
        Self::new_unchecked(slot)
    }
}