use crate::adi::{
    AdiError,
    AdiSlot
};

use pros_sys::{
    PROS_ERR,
    adi_ultrasonic_t
};

use crate::error::bail_on;

type ext_adi_u8_tuple_t = (u8, u8);
type ext_adi_port_tuple_t = (AdiSlot, AdiSlot);

pub struct AdiUltrasonic {
    tup: ext_adi_u8_tuple_t,
    reference: adi_ultrasonic_t,
}

impl AdiUltrasonic {
    pub unsafe fn new(tup: ext_adi_port_tuple_t) -> Self {
        let port_top = tup.0 as u8;
        let port_bottom = tup.1 as u8;
        Self {
            tup: (port_top, port_bottom),
            reference: pros_sys::adi_ultrasonic_init(port_top, port_bottom),
        }
    }

    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_ultrasonic_get(self.reference)) })
    }

    pub fn shutdown(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_ultrasonic_shutdown(self.reference)) })
    }
}