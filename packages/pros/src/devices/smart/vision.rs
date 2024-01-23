//! Vision sensor device.
//!
//! Vision sensors take in a zero point at creation.

extern crate alloc;
use core::num::NonZeroU8;

use alloc::vec::Vec;

use pros_sys::{PROS_ERR, VISION_OBJECT_ERR_SIG};
use snafu::Snafu;

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::{
    error::{bail_on, map_errno, PortError},
    lvgl::colors::LcdColor,
};

/// Represents a vision sensor plugged into a smart port.
#[derive(Debug, Eq, PartialEq)]
pub struct VisionSensor {
    port: SmartPort,
    origin_point: VisionOriginPoint,
}

impl VisionSensor {
    /// Creates a new vision sensor on a smart port.
    pub fn new(port: SmartPort, origin_point: VisionOriginPoint) -> Result<Self, VisionError> {
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::vision_set_zero_point(port.index(), origin_point as _)
            );
        }

        Ok(Self {
            port,
            origin_point: origin_point,
        })
    }

    /// Adds a detection signature to the sensor's onboard memory. This signature will be used to
    /// identify objects when using [`Self::objects`].
    ///
    /// The sensor can store up to 7 unique signatures, with each signature slot denoted by the
    /// [`VisionSignature::id`] field.
    ///
    /// # Volatile Memory
    ///
    /// The memory on the Vision Sensor is *volatile* and will therefore be wiped when the sensor
    /// loses power. As a result, this function should be called every time the sensor is used on
    /// program start.
    pub fn add_signature(&self, signature: VisionSignature) -> Result<(), VisionError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::vision_set_signature(self.port.index(), signature.id.get(), &signature.into())
        });

        Ok(())
    }

    /// Adds a color code to the sensor's onboard memory. This code will be used to identify objects
    /// when using [`Self::objects`].
    ///
    /// Color codes are effectively "signature groups" that the sensor will use to identify objects
    /// containing the color of their signatures next to each other.
    ///
    /// # Volatile Memory
    ///
    /// The onboard memory of the Vision Sensor is *volatile* and will therefore be wiped when the
    /// sensor loses its power source. As a result, this function should be called every time the
    /// sensor is used on program start.
    pub fn add_code(&self, code: VisionCode) -> Result<(), VisionError> {
        _ = bail_on!(VISION_OBJECT_ERR_SIG, unsafe {
            pros_sys::vision_create_color_code(
                self.port.index(),
                code.sig_1.id.get() as u32,
                code.sig_2.id.get() as u32,
                if let Some(sig_3) = code.sig_3 { sig_3.id.get() } else { 0 } as u32,
                if let Some(sig_4) = code.sig_4 { sig_4.id.get() } else { 0 } as u32,
                if let Some(sig_5) = code.sig_5 { sig_5.id.get() } else { 0 } as u32,
            )
        });

        Ok(())
    }

    /// Get the current exposure percentage of the vision sensor.
    ///
    /// The returned result should be within 0.0 to 1.5.
    pub fn exposure(&self) -> Result<f32, VisionError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::vision_get_exposure(self.port.index())
        }) as f32
            * 1.5
            / 150.0)
    }

    /// Get the current white balance of the vision sensor as an RGB color.
    pub fn current_white_balance(&self) -> Result<Rgb, VisionError> {
        Ok((bail_on!(PROS_ERR, unsafe {
            pros_sys::vision_get_white_balance(self.port.index())
        }) as u32)
            .into())
    }

    /// Sets the exposure percentage of the vision sensor. Should be between 0.0 and 1.5.
    pub fn set_exposure(&mut self, exposure: f32) -> Result<(), VisionError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::vision_set_exposure(self.port.index(), (exposure * 150.0 / 1.5) as u8)
        });

        Ok(())
    }

    /// Sets the white balance of the vision sensor.
    ///
    /// White balance can be either automatically set or manually set through an RGB color.
    pub fn set_white_balance(&mut self, white_balance: WhiteBalance) -> Result<(), VisionError> {
        match white_balance {
            WhiteBalance::Auto => {
                bail_on!(PROS_ERR, unsafe {
                    pros_sys::vision_set_auto_white_balance(self.port.index(), 1)
                });
            }
            WhiteBalance::Rgb(rgb) => {
                // Turn off automatic white balance, since the user wants to do this manually.
                bail_on!(PROS_ERR, unsafe {
                    pros_sys::vision_set_auto_white_balance(self.port.index(), 0)
                });

                // Set manual RGB white balance.
                bail_on!(PROS_ERR, unsafe {
                    pros_sys::vision_set_white_balance(
                        self.port.index(),
                        <Rgb as Into<u32>>::into(rgb) as i32,
                    )
                });
            }
        };

        Ok(())
    }

    /// Configure the behavior of the LED indicator on the sensor.
    ///
    /// The default behavior is represented by [`LedMode::Auto`], which will display the color of the most prominent
    /// detected object's signature color. Alternatively, the LED can be configured to display a single RGB color.
    pub fn set_led_mode(&mut self, mode: LedMode) -> Result<(), VisionError> {
        match mode {
            LedMode::Auto => bail_on!(PROS_ERR, unsafe {
                pros_sys::vision_clear_led(self.port.index())
            }),
            LedMode::Rgb(rgb) => bail_on!(PROS_ERR, unsafe {
                pros_sys::vision_set_led(self.port.index(), <Rgb as Into<u32>>::into(rgb) as i32)
            }),
        };
        Ok(())
    }

    /// Sets the point that object positions are relative to.
    ///
    /// In other words, this function will change where (0, 0) is located in the sensor's coordinate system.
    pub fn set_origin_point(&mut self, origin: VisionOriginPoint) -> Result<(), VisionError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::vision_set_zero_point(self.port.index(), origin as _)
        });

        self.origin_point = origin;

        Ok(())
    }

    pub fn origin_point(&self) -> VisionOriginPoint {
        self.origin_point
    }

    /// Gets a list of objects detected by the sensor ordered from largest to smallest in size.
    pub fn objects(&self) -> Result<Vec<VisionObject>, VisionError> {
        let object_count = self.object_count()?;
        let mut objects = Vec::with_capacity(object_count);

        bail_on!(PROS_ERR, unsafe {
            pros_sys::vision_read_by_size(
                self.port.index(),
                0,
                object_count as u32,
                objects.as_mut_ptr(),
            )
        });

        Ok(objects
            .into_iter()
            .filter_map(|object| object.try_into().ok())
            .collect())
    }

    /// Returns the number of objects detected by the sensor.
    pub fn object_count(&self) -> Result<usize, VisionError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::vision_get_object_count(self.port.index())
        }) as usize)
    }
}

impl SmartDevice for VisionSensor {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Vision
    }
}

pub struct VisionSignature {
    /// The signature id.
    id: NonZeroU8,

    /// The (min, max, mean) values on the u axis.
    u_threshold: (i32, i32, i32),

    /// The (min, max, mean) values on the v axis.
    v_threshold: (i32, i32, i32),

    /// The signature range scale factor.
    range: f32,

    /// The RGB8 color of the signature.
    rgb: Rgb,

    /// The signature type, normal or color code.
    signature_type: VisionSignatureType,
}

impl VisionSignature {
    pub fn new(
        id: NonZeroU8,
        u_threshold: (i32, i32, i32),
        v_threshold: (i32, i32, i32),
        range: f32,
        rgb: Rgb,
        signature_type: VisionSignatureType,
    ) -> Self {
        Self {
            id,
            u_threshold,
            v_threshold,
            range,
            rgb,
            signature_type,
        }
    }

    pub fn from_utility(
        id: u8,
        u_min: i32,
        u_max: i32,
        u_mean: i32,
        v_min: i32,
        v_max: i32,
        v_mean: i32,
        range: f32,
        rgb: u32,
        signature_type: u32,
    ) -> Self {
        Self {
            id: NonZeroU8::new(id).expect("Vision signature IDs must not be 0"),
            u_threshold: (u_min, u_max, u_mean),
            v_threshold: (v_min, v_max, v_mean),
            range,
            rgb: rgb.into(),
            signature_type: signature_type.into(),
        }
    }
}

impl TryFrom<pros_sys::vision_signature_s_t> for VisionSignature {
    type Error = VisionError;

    fn try_from(value: pros_sys::vision_signature_s_t) -> Result<Self, VisionError> {
        Ok(Self {
            id: NonZeroU8::new(bail_on!(VISION_OBJECT_ERR_SIG as u8, value.id)).expect("Vision signature IDs must not be 0"),
            u_threshold: (value.u_min, value.u_max, value.u_mean),
            v_threshold: (value.v_min, value.v_max, value.v_mean),
            range: value.range,
            rgb: value.rgb.into(),
            signature_type: value.r#type.into(),
        })
    }
}

impl From<VisionSignature> for pros_sys::vision_signature_s_t {
    fn from(value: VisionSignature) -> pros_sys::vision_signature_s_t {
        pros_sys::vision_signature_s_t {
            id: value.id.get(),
            _pad: [0; 3],
            u_min: value.u_threshold.0,
            u_max: value.u_threshold.1,
            u_mean: value.u_threshold.2,
            v_min: value.v_threshold.0,
            v_max: value.v_threshold.1,
            v_mean: value.v_threshold.2,
            range: value.range,
            rgb: value.rgb.into(),
            r#type: value.signature_type.into(),
        }
    }
}

pub struct VisionCode {
    pub sig_1: VisionSignature,
    pub sig_2: VisionSignature,
    pub sig_3: Option<VisionSignature>,
    pub sig_4: Option<VisionSignature>,
    pub sig_5: Option<VisionSignature>,
}

type TwoSignatures = (VisionSignature, VisionSignature);
type ThreeSignatures = (VisionSignature, VisionSignature, VisionSignature);
type FourSignatures = (
    VisionSignature,
    VisionSignature,
    VisionSignature,
    VisionSignature,
);
type FiveSignatures = (
    VisionSignature,
    VisionSignature,
    VisionSignature,
    VisionSignature,
    VisionSignature,
);

impl VisionCode {
    pub fn new(
        sig_1: VisionSignature,
        sig_2: VisionSignature,
        sig_3: Option<VisionSignature>,
        sig_4: Option<VisionSignature>,
        sig_5: Option<VisionSignature>,
    ) -> Self {
        Self {
            sig_1,
            sig_2,
            sig_3,
            sig_4,
            sig_5,
        }
    }
}

impl From<TwoSignatures> for VisionCode {
    fn from(signatures: TwoSignatures) -> Self {
        Self {
            sig_1: signatures.0,
            sig_2: signatures.1,
            sig_3: None,
            sig_4: None,
            sig_5: None,
        }
    }
}

impl From<ThreeSignatures> for VisionCode {
    fn from(signatures: ThreeSignatures) -> Self {
        Self {
            sig_1: signatures.0,
            sig_2: signatures.1,
            sig_3: Some(signatures.2),
            sig_4: None,
            sig_5: None,
        }
    }
}

impl From<FourSignatures> for VisionCode {
    fn from(signatures: FourSignatures) -> Self {
        Self {
            sig_1: signatures.0,
            sig_2: signatures.1,
            sig_3: Some(signatures.2),
            sig_4: Some(signatures.3),
            sig_5: None,
        }
    }
}

impl From<FiveSignatures> for VisionCode {
    fn from(signatures: FiveSignatures) -> Self {
        Self {
            sig_1: signatures.0,
            sig_2: signatures.1,
            sig_3: Some(signatures.2),
            sig_4: Some(signatures.3),
            sig_5: Some(signatures.4),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionSignatureType {
    Normal = pros_sys::E_VISION_OBJECT_NORMAL,
    ColorCode = pros_sys::E_VISION_OBJECT_COLOR_CODE,
    Line = pros_sys::E_VISION_OBJECT_LINE,
}

impl From<VisionSignatureType> for pros_sys::vision_object_type_e_t {
    fn from(value: VisionSignatureType) -> pros_sys::vision_object_type_e_t {
        value as _
    }
}

impl From<pros_sys::vision_object_type_e_t> for VisionSignatureType {
    fn from(value: pros_sys::vision_object_type_e_t) -> VisionSignatureType {
        match value {
            pros_sys::E_VISION_OBJECT_NORMAL => Self::Normal,
            pros_sys::E_VISION_OBJECT_COLOR_CODE => Self::ColorCode,
            pros_sys::E_VISION_OBJECT_LINE => Self::Line,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisionObject {
    pub signature_id: u16,
    pub signature_type: VisionSignatureType,

    pub x: i16,
    pub y: i16,

    pub center_x: i16,
    pub center_y: i16,

    pub angle: i16,

    pub width: i16,
    pub height: i16,
}

impl TryFrom<pros_sys::vision_object_s_t> for VisionObject {
    type Error = VisionError;

    fn try_from(value: pros_sys::vision_object_s_t) -> Result<Self, VisionError> {
        Ok(Self {
            signature_id: bail_on!(VISION_OBJECT_ERR_SIG, value.signature),
            signature_type: value.r#type.into(),

            x: value.top_coord,
            y: value.left_coord,

            center_x: value.x_middle_coord,
            center_y: value.y_middle_coord,

            angle: value.angle,

            width: value.width,
            height: value.height,
        })
    }
}

impl From<VisionObject> for pros_sys::vision_object_s_t {
    fn from(value: VisionObject) -> pros_sys::vision_object_s_t {
        pros_sys::vision_object_s_t {
            signature: value.signature_id,
            r#type: value.signature_type.into(),
            left_coord: value.x,
            top_coord: value.y,
            width: value.width,
            height: value.height,
            angle: value.angle,
            x_middle_coord: value.center_x,
            y_middle_coord: value.center_y,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<Rgb> for u32 {
    fn from(other: Rgb) -> u32 {
        ((other.r as u32) << 16) + ((other.g as u32) << 8) + other.b as u32
    }
}

const BITMASK: u32 = 0b11111111;

impl From<u32> for Rgb {
    fn from(value: u32) -> Self {
        Self {
            r: ((value >> 16) & BITMASK) as _,
            g: ((value >> 8) & BITMASK) as _,
            b: (value & BITMASK) as _,
        }
    }
}

impl From<Rgb> for LcdColor {
    fn from(other: Rgb) -> Self {
        Self(pros_sys::lv_color_t {
            red: other.r,
            green: other.g,
            blue: other.b,
            alpha: 0xFF,
        })
    }
}

impl From<LcdColor> for Rgb {
    fn from(other: LcdColor) -> Self {
        Self {
            r: other.red,
            g: other.green,
            b: other.blue,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionOriginPoint {
    TopLeft = pros_sys::E_VISION_ZERO_TOPLEFT,
    Center = pros_sys::E_VISION_ZERO_CENTER,
}

impl From<VisionOriginPoint> for pros_sys::vision_zero_e_t {
    fn from(value: VisionOriginPoint) -> pros_sys::vision_zero_e_t {
        value as _
    }
}

impl From<pros_sys::vision_zero_e_t> for VisionOriginPoint {
    fn from(value: pros_sys::vision_zero_e_t) -> VisionOriginPoint {
        match value {
            pros_sys::E_VISION_ZERO_TOPLEFT => Self::TopLeft,
            pros_sys::E_VISION_ZERO_CENTER => Self::Center,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteBalance {
    Rgb(Rgb),
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedMode {
    Rgb(Rgb),
    Auto,
}

#[derive(Debug, Snafu)]
pub enum VisionError {
    #[snafu(display(
        "The index specified was higher than the total number of objects seen by the camera."
    ))]
    IndexTooHigh,
    #[snafu(display(
        "The given signature ID or argument is out of range."
    ))]
    InvalidIdentifier,
    #[snafu(display("The camera could not be read."))]
    ReadingFailed,
    #[snafu(display("Another resource is currently trying to access the port."))]
    ConcurrentAccess,
    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
}

map_errno! {
    VisionError {
        EHOSTDOWN | EAGAIN => Self::ReadingFailed,
        EDOM => Self::IndexTooHigh,
        EINVAL => Self::InvalidIdentifier,
        EACCES => Self::ConcurrentAccess,
    }
    inherit PortError;
}
