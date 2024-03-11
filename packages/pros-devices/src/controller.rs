//! Read from the buttons and joysticks on the controller and write to the controller's display.
//!
//! Controllers are identified by their id, which is either 0 (master) or 1 (partner).
//! State of a controller can be checked by calling [`Controller::state`] which will return a struct with all of the buttons' and joysticks' state.

use alloc::ffi::CString;

use pros_core::{bail_on, map_errno};
use pros_sys::{E_CONTROLLER_MASTER, E_CONTROLLER_PARTNER, PROS_ERR};
use snafu::Snafu;
use crate::adi::digital::LogicLevel;

/// Digital Controller Button
#[derive(Debug, Eq, PartialEq)]
pub struct Button {
    id: ControllerId,
    channel: pros_sys::controller_digital_e_t,
}

impl Button {
    /// Gets the current logic level of a digital input pin.
    pub fn level(&self) -> Result<LogicLevel, ControllerError> {
        let value = bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_digital(self.id as _, self.channel)
        }) != 0;

        Ok(match value {
            true => LogicLevel::High,
            false => LogicLevel::Low,
        })
    }

    /// Returrns `true` if the button is currently being pressed.
    ///
    /// This is equivalent shorthand to calling `Self::level().is_high()`.
    pub fn is_pressed(&self) -> Result<bool, ControllerError> {
        Ok(self.level()?.is_high())
    }


    /// Returns `true` if the button has been pressed again since the last time this
    /// function was called.
    ///
    /// # Thread Safety
    ///
    /// This function is not thread-safe.
    ///
    /// Multiple tasks polling a single button may return different results under the
    /// same circumstances, so only one task should call this function for any given
    /// switch. E.g., Task A calls this function for buttons 1 and 2. Task B may call
    /// this function for button 3, but should not for buttons 1 or 2. A typical
    /// use-case for this function is to call inside opcontrol to detect new button
    /// presses, and not in any other tasks.
    pub fn was_pressed(&mut self) -> Result<bool, ControllerError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_get_digital_new_press(self.id as _, self.channel)
        }) == 1)
    }
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

/// The basic type for a controller.
/// Used to get the state of its joysticks and controllers.
#[derive(Debug, Eq, PartialEq)]
pub struct Controller {
    id: ControllerId,

    pub screen: ControllerScreen,

    pub left_stick: Joystick,
    pub right_stick: Joystick,

    pub button_a: Button,
    pub button_b: Button,
    pub button_x: Button,
    pub button_y: Button,
    pub button_up: Button,
    pub button_down: Button,
    pub button_left: Button,
    pub button_right: Button,

    pub left_trigger_1: Button,
    pub left_trigger_2: Button,
    pub right_trigger_1: Button,
    pub right_trigger_2: Button,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ControllerScreen {
    id: ControllerId,
}

impl ControllerScreen {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ControllerId {
    Master = E_CONTROLLER_MASTER,
    Partner = E_CONTROLLER_PARTNER,
}

impl Controller {
    pub const MAX_LINE_LENGTH: usize = 14;
    pub const MAX_LINES: usize = 2;

    pub const unsafe fn new(id: ControllerId) -> Self {
        Self {
            id,
            screen: ControllerScreen { id },
            left_stick: Joystick {
                id,
                x_channel: pros_sys::E_CONTROLLER_ANALOG_LEFT_X,
                y_channel: pros_sys::E_CONTROLLER_ANALOG_LEFT_Y,
            },
            right_stick: Joystick {
                id,
                x_channel: pros_sys::E_CONTROLLER_ANALOG_RIGHT_X,
                y_channel: pros_sys::E_CONTROLLER_ANALOG_RIGHT_Y,
            },
            button_a: Button {
                id,
                channel: pros_sys::E_CONTROLLER_DIGITAL_A,
            },
            button_b: Button {
                id,
                channel: pros_sys::E_CONTROLLER_DIGITAL_B,
            },
            button_x: Button {
                id,
                channel: pros_sys::E_CONTROLLER_DIGITAL_X,
            },
            button_y: Button {
                id,
                channel: pros_sys::E_CONTROLLER_DIGITAL_Y,
            },
            button_up: Button {
                id,
                channel: pros_sys::E_CONTROLLER_DIGITAL_UP,
            },
            button_down: Button {
                id,
                channel: pros_sys::E_CONTROLLER_DIGITAL_DOWN,
            },
            button_left: Button {
                id,
                channel: pros_sys::E_CONTROLLER_DIGITAL_LEFT,
            },
            button_right: Button {
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
        }
    }

    pub fn is_connected(&self) -> Result<bool, ControllerError> {
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
