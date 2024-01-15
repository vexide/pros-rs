//! Distance sensor device.
//!
//! Pretty much one to one with the PROS C and CPP API, except Result is used instead of ERRNO values.

use core::ffi::c_double;

use pros_sys::PROS_ERR;
use uom::si::{
    length::millimeter,
    quantities::{Length, Ratio, Velocity},
    ratio::ratio,
    velocity::meter_per_second,
};

use crate::error::{bail_on, PortError};

/// A physical distance sensor plugged into a port.
/// Distance sensors can only keep track of one object at a time.
pub struct DistanceSensor {
    port: u8,
}

impl DistanceSensor {
    pub fn new(port: u8) -> Result<Self, PortError> {
        let sensor = Self { port };
        sensor.distance()?;
        Ok(sensor)
    }

    /// Returns the distance to the object the sensor detects in millimeters.
    pub fn distance(&self) -> Result<Length<f64>, PortError> {
        let raw = unsafe { bail_on!(PROS_ERR, pros_sys::distance_get(self.port)) };
        Ok(Length::new::<millimeter>(raw.into()))
    }

    /// returns the velocity of the object the sensor detects in m/s
    pub fn object_velocity(&self) -> Result<Velocity<f64>, PortError> {
        // all VEX Distance Sensor functions return PROS_ERR on failure even though
        // some return floating point values (not PROS_ERR_F)
        let raw = unsafe {
            bail_on!(
                PROS_ERR as c_double,
                pros_sys::distance_get_object_velocity(self.port)
            )
        };
        Ok(Velocity::new::<meter_per_second>(raw))
    }

    /// Returns the confidence of the distance measurement.
    pub fn distance_confidence(&self) -> Result<Ratio<f64>, PortError> {
        // 0 -> 63
        let raw = unsafe { bail_on!(PROS_ERR, pros_sys::distance_get_confidence(self.port)) };
        Ok(Ratio::new::<ratio>(raw as f64 / 63.0))
    }
}
