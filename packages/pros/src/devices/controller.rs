//! Read from the buttons and joysticks on the controller and write to the controller's display.
//!
//! Controllers are identified by their id, which is either 0 (master) or 1 (partner).
//! State of a controller can be checked by calling [`Controller::state`] which will return a struct with all of the buttons' and joysticks' state.

use alloc::ffi::CString;

use pros_sys::{E_CONTROLLER_MASTER, E_CONTROLLER_PARTNER, PROS_ERR};
use snafu::Snafu;

use crate::error::{bail_on, map_errno};

pub const CONTROLLER_MAX_LINE_LENGTH: usize = 14;
pub const CONTROLLER_MAX_LINES: usize = 2;

#[derive(Debug, Eq, PartialEq)]
pub struct Button {
    id: ControllerId,
    channel: pros_sys::controller_digital_e_t,
}

impl Button {
    pub fn pressed(&self) -> Result<bool, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_digital(self.id as _, self.channel)
        }) == 1)
    }

    pub fn pressed_again(&mut self) -> Result<bool, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_digital_new_press(self.id as _, self.channel)
        }) == 1)
    }
}

/// Holds whether or not the buttons on the controller are pressed or not
#[derive(Debug, Eq, PartialEq)]
pub struct Buttons {
    /// The 'A' button on the right button pad of the controller.
    pub a: Button,
    /// The 'B' button on the right button pad of the controller.
    pub b: Button,
    /// The 'X' button on the right button pad of the controller.
    pub x: Button,
    /// The 'Y' button on the right button pad of the controller.
    pub y: Button,

    /// The up arrow on the left arrow pad of the controller.
    pub up: Button,
    /// The down arrow on the left arrow pad of the controller.
    pub down: Button,
    /// The left arrow on the left arrow pad of the controller.
    pub left: Button,
    /// The right arrow on the left arrow pad of the controller.
    pub right: Button,

    /// The top trigger on the left side of the controller.
    pub left_trigger_1: Button,
    /// The bottom trigger on the left side of the controller.
    pub left_trigger_2: Button,
    /// The top trigger on the right side of the controller.
    pub right_trigger_1: Button,
    /// The bottom trigger on the right side of the controller.
    pub right_trigger_2: Button,
}

/// Stores how far the joystick is away from the center (at *(0, 0)*) from -1 to 1.
/// On the x axis left is negative, and right is positive.
/// On the y axis down is negative, and up is positive.
#[derive(Debug, Eq, PartialEq)]
pub struct Joystick {
    id: ControllerId,
    x_channel: pros_sys::controller_analog_e_t,
    y_channel: pros_sys::controller_analog_e_t,
}

impl Joystick {
    pub fn x(&self) -> Result<f32, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_analog(self.id as _, self.x_channel)
        }) as f32
            / 127.0)
    }

    pub fn y(&self) -> Result<f32, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_analog(self.id as _, self.y_channel)
        }) as f32
            / 127.0)
    }

    pub fn x_raw(&self) -> Result<i8, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_analog(self.id as _, self.x_channel)
        }) as _)
    }

    pub fn y_raw(&self) -> Result<i8, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_analog(self.id as _, self.y_channel)
        }) as _)
    }
}

/// Stores both joysticks on the controller.
#[derive(Debug, Eq, PartialEq)]
pub struct Joysticks {
    /// Left joystick
    pub left: Joystick,
    /// Right joystick
    pub right: Joystick,
}

/// The basic type for a controller.
/// Used to get the state of its joysticks and controllers.
#[derive(Debug, Eq, PartialEq)]
pub struct Controller {
    id: ControllerId,
    pub joysticks: Joysticks,
    /// Digital buttons state
    pub buttons: Buttons,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ControllerId {
    Master = E_CONTROLLER_MASTER,
    Partner = E_CONTROLLER_PARTNER,
}

impl Controller {
    pub const unsafe fn new(id: ControllerId) -> Self {
        Self {
            id,
            joysticks: Joysticks {
                left: Joystick {
                    id,
                    x_channel: pros_sys::E_CONTROLLER_ANALOG_LEFT_X,
                    y_channel: pros_sys::E_CONTROLLER_ANALOG_LEFT_Y,
                },
                right: Joystick {
                    id,
                    x_channel: pros_sys::E_CONTROLLER_ANALOG_RIGHT_X,
                    y_channel: pros_sys::E_CONTROLLER_ANALOG_RIGHT_Y,
                },
            },
            buttons: Buttons {
                a: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_A,
                },
                b: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_B,
                },
                x: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_X,
                },
                y: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_Y,
                },
                up: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_UP,
                },
                down: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_DOWN,
                },
                left: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_LEFT,
                },
                right: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_RIGHT,
                },
                left_trigger_1: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_L1,
                },
                left_trigger_2: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_L2,
                },
                right_trigger_1: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_R2,
                },
                right_trigger_2: Button {
                    id,
                    channel: pros_sys::E_CONTROLLER_DIGITAL_R2,
                },
            },
        }
    }

    pub fn connected(&self) -> Result<bool, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_is_connected(self.id as _)
        }) != 0)
    }

    pub fn battery_capacity(&self) -> Result<i32, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_battery_capacity(self.id as _)
        }))
    }

    pub fn battery_level(&self) -> Result<i32, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_battery_level(self.id as _)
        }))
    }

    pub fn rumble(&mut self, pattern: &str) -> Result<(), ControllerError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_rumble(
                self.id as _,
                CString::new(pattern)
                    .map_err(|_| ControllerError::NonTerminatingNul)?
                    .into_raw(),
            )
        });

        Ok(())
    }

    pub fn clear_line(&mut self, line: u8) -> Result<(), ControllerError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_clear_line(self.id as _, line)
        });

        Ok(())
    }

    pub fn clear_screen(&mut self) -> Result<(), ControllerError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_clear(self.id as _)
        });

        Ok(())
    }

    pub fn set_text(&mut self, text: &str, line: u8, col: u8) -> Result<(), ControllerError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_set_text(
                self.id as _,
                line,
                col,
                CString::new(text)
                    .map_err(|_| ControllerError::NonTerminatingNul)?
                    .into_raw(),
            )
        });

        Ok(())
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with the controller.
pub enum ControllerError {
    /// The controller ID given was invalid, expected E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER.
    InvalidControllerId,

    /// Another resource is already using the controller.
    ConcurrentAccess,

    /// CString::new encountered NULL (U+0000) byte in non-terminating position.
    NonTerminatingNul,
}

map_errno! {
    ControllerError {
        EACCES => Self::ConcurrentAccess,
        EINVAL => Self::InvalidControllerId,
    }
}
