//! An embedded_graphics driver for VEX V5 Brain displays.
//! Implemented for the [`pros-rs`](https://crates.io/crates/pros) ecosystem and implemented using [pros-devices](https://crates.io/crates/pros-devices).
#![no_std]
#![feature(new_uninit)]

extern crate alloc;

use alloc::boxed::Box;

use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{Dimensions, Point, Size},
    pixelcolor::{Rgb888, RgbColor},
    primitives::Rectangle,
    Pixel,
};
use pros_devices::{color::Rgb, Screen};

/// An embedded_graphics driver for the Brain display
pub struct VexDisplay {
    screen: Screen,
    pixel_buffer:
        Box<[u32; Screen::HORIZONTAL_RESOLUTION as usize * Screen::VERTICAL_RESOLUTION as usize]>,
}

impl VexDisplay {
    /// Creates a new VexDisplay from a Screen
    pub fn new(screen: Screen) -> Self {
        let pixel_buffer = Box::new_zeroed();
        let pixel_buffer = unsafe { pixel_buffer.assume_init() };

        Self {
            screen,
            pixel_buffer,
        }
    }

    unsafe fn draw_buffer(&self) {
        unsafe {
            pros_sys::screen_copy_area(
                0,
                0,
                Screen::HORIZONTAL_RESOLUTION,
                Screen::VERTICAL_RESOLUTION,
                self.pixel_buffer.as_ptr(),
                Screen::HORIZONTAL_RESOLUTION as _,
            );
        }
    }
}

impl From<Screen> for VexDisplay {
    fn from(value: Screen) -> Self {
        Self::new(value)
    }
}

impl Dimensions for VexDisplay {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(
            Point::new(0, 0),
            Size::new(
                Screen::HORIZONTAL_RESOLUTION as _,
                Screen::VERTICAL_RESOLUTION as _,
            ),
        )
    }
}

impl DrawTarget for VexDisplay {
    type Color = Rgb888;
    type Error = pros_devices::screen::ScreenError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        pixels
            .into_iter()
            .map(|pixel| (pixel.0, Rgb::new(pixel.1.r(), pixel.1.g(), pixel.1.b())))
            .for_each(|(pos, color)| {
                // Make sure that the coordinate is valid to index with.
                if !(pos.x > Screen::HORIZONTAL_RESOLUTION as _ || pos.x < 0)
                    && !(pos.y > Screen::VERTICAL_RESOLUTION as _ || pos.y < 0)
                {
                    // SAFETY: We initialize the buffer with zeroes, so it's safe to assume it's initialized.
                    self.pixel_buffer[(pos.y as usize * Screen::HORIZONTAL_RESOLUTION as usize)
                        + pos.x as usize] = color.into();
                }
            });

        unsafe {
            self.draw_buffer();
        }
        Ok(())
    }
}
