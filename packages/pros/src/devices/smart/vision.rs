//! Vision sensor device module.
//!
//! This module provides an interface for interacting with the VEX Vision Sensor.
//!
//! # Hardware Overview
//!
//! The VEX Vision Sensor is a device powered by a standalone ARM Cortex M4+M0 processor
//! and color camera for the purpose of performing object recognition. The sensor can be
//! trained to locate objects by color. The camera module itself is very similar internally
//! to the Pixy2 camera, and performs its own onboard image processing. Manually processing
//! raw image data from the sensor is not currently possible.
//!
//! Every 200 milliseconds, the camera provides a list of the objects found matching up
//! to seven unique [`VisionSignature`]s. The object’s height, width, and location is provided.
//! Multi-colored objects may also be programmed through the use of [`ColorCode`]s.
//!
//! The Vision Sensor has USB for a direct connection to a computer, where it can be configured
//! using VEX's proprietary vision utility tool to generate color signatures. The Vision Sensor
//! also has WiFi Direct and can act as web server, allowing a live video feed of the camera
//! from any computer equipped with a browser and WiFi.

extern crate alloc;
use alloc::vec::Vec;
use core::{num::NonZeroU8, time::Duration};

use pros_sys::{PROS_ERR, VISION_OBJECT_ERR_SIG};
use snafu::Snafu;

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::{
    error::{bail_on, map_errno, PortError},
    lvgl::colors::LcdColor,
};

/// The horizontal resolution of the vision sensor.
///
/// This value is based on the `VISION_FOV_WIDTH` macro constant in PROS.
pub const VISION_RESOLUTION_WIDTH: u16 = 316;

/// The vertical resolution of the vision sensor.
///
/// This value is based on the `VISION_FOV_HEIGHT` macro constant in PROS.
pub const VISION_RESOLUTION_HEIGHT: u16 = 212;

/// The update rate of the vision sensor.
pub const VISION_UPDATE_RATE: Duration = Duration::from_millis(50);

/// VEX Vision Sensor
///
/// This struct represents a vision sensor plugged into a smart port.
#[derive(Debug, Eq, PartialEq)]
pub struct VisionSensor {
    port: SmartPort,
    origin_point: VisionOriginPoint,
}

impl VisionSensor {
    /// Creates a new vision sensor on a smart port.
    pub fn new(port: SmartPort, origin_point: VisionOriginPoint) -> Result<Self, VisionError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::vision_set_zero_point(port.index(), origin_point as pros_sys::vision_zero_e_t)
        });

        Ok(Self {
            port,
            origin_point: origin_point,
        })
    }

    /// Adds a detection signature to the sensor's onboard memory. This signature will be used to
    /// identify objects when using [`Self::objects`].
    ///
    /// The sensor can store up to 7 unique signatures, with each signature slot denoted by the
    /// [`VisionSignature::id`] field. If a signature with an ID matching an existing signature
    /// on the sensor is added, then the existing signature will be overwritten with the new one.
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
                if let Some(sig_3) = code.sig_3 {
                    sig_3.id.get()
                } else {
                    0
                } as u32,
                if let Some(sig_4) = code.sig_4 {
                    sig_4.id.get()
                } else {
                    0
                } as u32,
                if let Some(sig_5) = code.sig_5 {
                    sig_5.id.get()
                } else {
                    0
                } as u32,
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
            WhiteBalance::Manual(rgb) => {
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
            LedMode::Manual(rgb) => bail_on!(PROS_ERR, unsafe {
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

/// A vision detection color signature.
///
/// Vision signatures contain information used by the vision sensor to detect objects of a certain
/// color. These signatures are typically generated through VEX's vision utility tool rather than
/// written by hand. For creating signatures using the utility, see [`Self::from_utility`].
///
/// # Format & Detection Overview
///
/// Vision signatures operate in a version of the Y'UV color space, specifically using the "U" and "V"
/// chroma components for edge detection purposes. This can be seen in the `u_threshold` and
/// `v_threshold` fields of this struct. These fields place three "threshold" (min, max, mean)
/// values on the u and v chroma values detected by the sensor. The values are then transformed to a
/// 3D lookup table to detect actual colors.
///
/// There is additionally a `range` field, which works as a scale factor or threshold for how lenient
/// edge detection should be.
///
/// Signatures can additionally be grouped together into [`VisionCode`]s, which narrow the filter for
/// object detection by requiring two colors
pub struct VisionSignature {
    /// The signature id.
    ///
    /// This number will determine the slot that the signature is placed into when adding it
    /// to a [`VisionSensor`]'s onboard memory. This value ranges from 1-7. For more information,
    /// see [`VisionSensor::add_signature`].
    pub id: NonZeroU8,

    /// The (min, max, mean) values on the "U" axis.
    ///
    /// This defines a threshold of values for the sensor to match against a certain chroma in the
    /// Y'UV color space - speciailly on the U component.
    pub u_threshold: (i32, i32, i32),

    /// The (min, max, mean) values on the V axis.
    ///
    /// This defines a threshold of values for the sensor to match against a certain chroma in the
    /// Y'UV color space - speciailly on the "V" component.
    pub v_threshold: (i32, i32, i32),

    /// The signature range scale factor.
    ///
    /// This value effectively serves as a threshold for how lenient the sensor should be
    /// when detecting the edges of colors. This value ranges from 0-11 in Vision Utility.
    ///
    /// Higher values of `range` will increase the range of brightness that the sensor will
    /// consider to be part of the signature. Lighter/Darker shades of the signature's color
    /// will be detected more often.
    pub range: f32,

    /// The signature type. Color codes are internally stored as signatures by the sensor,
    /// meaning this value may be different if the detection signature is stored in a color
    /// code.
    pub signature_type: VisionSignatureType,
}

impl VisionSignature {
    pub fn new(
        id: NonZeroU8,
        u_threshold: (i32, i32, i32),
        v_threshold: (i32, i32, i32),
        range: f32,
        signature_type: VisionSignatureType,
    ) -> Self {
        Self {
            id,
            u_threshold,
            v_threshold,
            range,
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
        signature_type: u32,
    ) -> Self {
        Self {
            id: NonZeroU8::new(id)
                .expect("Vision utility produced a signature with an invalid ID of 0."),
            u_threshold: (u_min, u_max, u_mean),
            v_threshold: (v_min, v_max, v_mean),
            range,
            signature_type: signature_type.into(),
        }
    }
}

impl TryFrom<pros_sys::vision_signature_s_t> for VisionSignature {
    type Error = VisionError;

    fn try_from(value: pros_sys::vision_signature_s_t) -> Result<Self, VisionError> {
        Ok(Self {
            id: NonZeroU8::new(bail_on!(VISION_OBJECT_ERR_SIG as u8, value.id))
                .expect("Vision signature IDs must not be 0"),
            u_threshold: (value.u_min, value.u_max, value.u_mean),
            v_threshold: (value.v_min, value.v_max, value.v_mean),
            range: value.range,
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
            // This seems to be an SDK internal left in the PROS API. PROS leaves their `rgb` field` as 0 when calling
            // vision_signature_from_utility`, meaning the value likely only exists for telemetry purposes in getters.
            rgb: 0,
            r#type: value.signature_type.into(),
        }
    }
}

/// A vision detection code.
///
/// [`VisionCode`]s are a special type of detection signature that group multiple [`VisionSignature`]s
/// together. A [`VisionCode`] can associate 2-5 color signatures together, detecting the resulting object
/// when its color signatures are present close to each other.
///
/// These codes work very similarly to [Pixy2 Color Codes](https://docs.pixycam.com/wiki/doku.php?id=wiki:v2:using_color_codes).
pub struct VisionCode {
    /// The first signature in the code.
    pub sig_1: VisionSignature,

    /// The second signature in the code.
    pub sig_2: VisionSignature,

    /// The third signature in the code.
    ///
    /// This signature is optional, and can be omitted if necessary.
    pub sig_3: Option<VisionSignature>,

    /// The fourth signature in the code.
    ///
    /// This signature is optional, and can be omitted if necessary.
    pub sig_4: Option<VisionSignature>,

    /// The fifth signature in the code.
    ///
    /// This signature is optional, and can be omitted if necessary.
    pub sig_5: Option<VisionSignature>,
}

// Type definitions to make this part less painful.

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
    /// Creates a new vision code.
    ///
    /// Two signatures are require to create a vision code, while the other three
    /// are optional.
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

/// A detected vision object.
///
/// This struct contains metadata about objects detected by the vision sensor. Objects are
/// detected by calling [`VisionSensor::objects`] after adding signatures and color codes
/// to the sensor.
///
/// # Coordinate System
///
/// The coordinate system used by the `x`, `y`, `center_x`, and `center_y` are dependent on
/// the [`VisionOriginPoint`] value passed to [`VisionSensor::new`] or [`VisionSensor::set_origin_point`].
///
/// - If the origin point is [`VisionOriginPoint::TopLeft`], then objects will use coordinates relaative
///   to the top left of the camera's field of view.
/// - If the origin point is [`VisionOriginPoint::Center`], then objects will use coordinates relaative
///   to the center left of the camera's field of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisionObject {
    /// The ID of the [`VisionSignature`] used to detect this object.
    pub signature_id: u16,

    /// The type of signature used to detect this object.
    pub signature_type: VisionSignatureType,

    /// The horizontal pixel offset from the specified [`VisionOriginPoint`] on the camera's field of view.
    pub x: i16,

    /// The vertical pixel offset from the specified [`VisionOriginPoint`] on the camera's field of view.
    pub y: i16,

    /// The horizontal pixel offset relative to the center of the object from the specified [`VisionOriginPoint`]
    /// on the camera's field of view.
    pub center_x: i16,

    /// The vertical pixel offset relative to the center of the object from the specified [`VisionOriginPoint`]
    /// on the camera's field of view.
    pub center_y: i16,

    /// The approximate degrees of rotation of the detected object's bounding box.
    pub angle: i16,

    /// The width of the detected object's bounding box in pixels.
    pub width: i16,

    /// The height of the detected object's bounding box in pixels.
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

/// Represents a 32-bit RGB color.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    /// The red component of the color.
    ///
    /// This value ranges from 0-255.
    pub r: u8,

    /// The green component of the color.
    ///
    /// This value ranges from 0-255.
    pub g: u8,

    /// The blue component of the color.
    ///
    /// This value ranges from 0-255.
    pub b: u8,
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

impl From<(u8, u8, u8)> for Rgb {
    fn from(tuple: (u8, u8, u8)) -> Rgb {
        Self {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
        }
    }
}

impl From<Rgb> for (u8, u8, u8) {
    fn from(value: Rgb) -> (u8, u8, u8) {
        (value.r, value.g, value.b)
    }
}

/// Defines an origin point for the coordinate system used by a vision sensor.
///
/// This value is passed to [`VisionSensor::new`] and [`VisionSensor::set_origin_point`]
/// and determines the origin point (0, 0) used by the sensor when reporting detected
/// objects.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionOriginPoint {
    /// The origin is relative to the top left of the camera's field of view.
    TopLeft = pros_sys::E_VISION_ZERO_TOPLEFT,

    /// The origin is relative to the center of the camera's field of view.
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

/// Vision Sensor white balance mode.
///
/// Represents a white balance configuration for the vision sensor's camera.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteBalance {
    /// Automatic Mode
    ///
    /// The sensor will automatically adjust the camera's white balance, using the brightest
    /// part of the image as a white point.
    #[default]
    Auto,

    /// Manual Mode
    ///
    /// Allows for manual control over white balance using an RGB color.
    Manual(Rgb),
}

/// Vision Sensor LED mode.
///
/// Represents the states that the integrated LED indicator on a vision sensor can be in.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedMode {
    /// Automatic Mode
    ///
    /// When in manual mode, the integrated LED will display a user-set RGB color code.
    #[default]
    Auto,

    /// Manual Mode
    ///
    /// When in automatic mode, the integrated LED will display the color of the most prominent
    /// detected object's signature color.
    Manual(Rgb),
}

#[derive(Debug, Snafu)]
pub enum VisionError {
    #[snafu(display(
        "The index specified was higher than the total number of objects seen by the camera."
    ))]
    IndexTooHigh,
    #[snafu(display("The given signature ID or argument is out of range."))]
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
