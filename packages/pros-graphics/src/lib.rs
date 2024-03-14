//! Graphics driver implementations for the V5 Brain display.
//!
//! Currently supports:
//!     - [embedded-graphics](https://crates.io/crates/embedded-graphics)
//! Implemented for the [`pros-rs`](https://crates.io/crates/pros) ecosystem and implemented using [pros-devices](https://crates.io/crates/pros-devices).
#![no_std]
#![cfg_attr(feature = "embedded-graphics", feature(new_uninit))]

#[cfg(feature = "embedded-graphics")]
extern crate alloc;

#[cfg(feature = "embedded-graphics")]
pub mod embedded_graphics;
