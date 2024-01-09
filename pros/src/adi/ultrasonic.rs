use crate::adi::{
    AdiError,
    port::AdiPort
};

use pros_sys::{
    PROS_ERR,
    adi_ultrasonic_t
};

use core::{ops::{
    Deref,
    DerefMut
}, borrow::Borrow};

use crate::error::bail_on;

type ext_adi_port_tuple_t = (*const AdiPort, *const AdiPort, *const AdiPort);

pub struct AdiUltrasonic {
    tup: ext_adi_port_tuple_t,
    reference: adi_ultrasonic_t,
}

impl AdiUltrasonic {
    pub unsafe fn new(tup: ext_adi_port_tuple_t) -> Self {
        Self {
            tup: tup,
            reference: pros_sys::adi_ultrasonic_init(*((**tup.0.borrow()).deref()).deref(), *(**tup.1.borrow().deref()).deref()),
        }
    }

    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_ultrasonic_get(self.reference)) })
    }

    pub fn shutdown(&mut self) -> Result<(), AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_ultrasonic_shutdown(self.reference)) })
    }
}

impl Deref for AdiUltrasonic {
    type Target = ext_adi_port_tuple_t;
    fn deref(&self) -> &Self::Target {
        &self.tup
    }
}

impl DerefMut for AdiUltrasonic {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tup
    }
}