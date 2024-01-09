
use core::{
    ffi::c_int,
    ops::{Deref, DerefMut},
};

use crate::adi::{
    AdiError,
    AdiSlot
};

use crate::error::bail_on;

use pros_sys::{PROS_ERR, adi_port_config_e_t, E_ADI_DIGITAL_IN, E_ADI_ANALOG_OUT, E_ADI_DIGITAL_OUT, E_ADI_ANALOG_IN, E_ADI_LEGACY_ENCODER, E_ADI_LEGACY_ULTRASONIC };

pub struct AdiPort(u8);

impl AdiPort {
    /// Create an AdiPort without checking if it is valid.
    ///
    /// # Safety
    ///
    /// The port must be above 0 and below [`pros_sys::NUM_ADI_PORTS`].
    pub unsafe fn new_unchecked(port: AdiSlot) -> Self {
        Self(port as u8)
    }
    /// Create an AdiPort, returning `None` if the port is invalid.
    pub fn try_new(slot: AdiSlot) -> Option<Self> {
        let port = slot as u8;
        if c_int::from(port) < pros_sys::NUM_ADI_PORTS {
            Some(Self(port))
        } else {
            None
        }
    }
    /// Create an AdiPort.
    ///
    /// # Panics
    ///
    /// Panics if the port is greater than or equal to [`pros_sys::NUM_ADI_PORTS`].
    pub fn new(port: AdiSlot) -> Self {
        Self::try_new(port).expect("Invalid ADI port")
    }

    pub fn set_value(&mut self, value: i32) -> Result<(), AdiError> {
        bail_on! {
            PROS_ERR,
            unsafe { pros_sys::adi_port_set_value(self.0, value) }
        }
        Ok(())
    }

    pub fn try_set_config(port: u8, config: adi_port_config_e_t) -> Result<(), AdiError> {
        if config == E_ADI_DIGITAL_IN || config == E_ADI_ANALOG_OUT || config == E_ADI_DIGITAL_OUT || config == E_ADI_ANALOG_IN || config == E_ADI_LEGACY_ENCODER || config == E_ADI_LEGACY_ULTRASONIC {
            bail_on! {
                PROS_ERR,
                unsafe { pros_sys::adi_port_set_config(port, config) }
            }
            Ok(())
        } else {
            Err(AdiError::InvalidConfigType)
        }
    }

    pub fn set_config(&mut self, config: adi_port_config_e_t) -> Result<(), AdiError> {
        bail_on! {
            PROS_ERR,
            unsafe { pros_sys::adi_port_set_config(self.0, config) }
        }
        Ok(())
    }

    pub fn config(&self) -> Result<adi_port_config_e_t, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_port_get_config(self.0)) })
    }
}

impl Deref for AdiPort {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AdiPort {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}