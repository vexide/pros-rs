use crate::adi::AdiError;

use pros_sys::{
    PROS_ERR,
    adi_ultrasonic_t
};

use crate::error::bail_on;

type ext_adi_port_tuple_t = (u8, u8);

pub struct AdiUltrasonic {
    tup: ext_adi_port_tuple_t,
    reference: adi_ultrasonic_t,
}

impl AdiUltrasonic {
    pub unsafe fn new(tup: ext_adi_port_tuple_t) -> Self {
        Self {
            tup,
            reference: pros_sys::adi_ultrasonic_init(tup.0, tup.1),
        }
    }

    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_ultrasonic_get(self.reference)) })
    }

    pub fn shutdown(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_ultrasonic_shutdown(self.reference)) })
    }
}