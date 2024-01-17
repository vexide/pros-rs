//! Helpers for dealing with errno.
//!
//! Most errors in pros-rs are created by reading the last value of ERRNO.
//! This includes the very generic [`PortError`], which is used for most hardware that gets plugged into a port on a V5 Brain.
//!
//! Most of the contents of this file are not public.

pub(crate) fn take_errno() -> i32 {
    let err = unsafe { *pros_sys::__errno() };
    if err != 0 {
        unsafe { *pros_sys::__errno() = 0 };
    }
    err
}

/// Generate an implementation of FromErrno for the given type.
///
/// Example:
/// ```ignore
/// map_errno! {
///     GpsError {
///         EAGAIN => Self::StillCalibrating,
///     }
///     inherit PortError;
/// }
/// ```
macro_rules! map_errno {
    {
        $err_ty:ty { $($errno:pat => $err:expr),*$(,)? }
        $(inherit $base:ty;)?
        $(unknown = $unknown:expr;)?
    } => {
        impl $crate::error::FromErrno for $err_ty {
            fn try_from_errno(num: i32) -> Option<Self> {
                #[allow(unused_imports)]
                use pros_sys::error::*;
                $(
                    // if the enum we're inheriting from can handle this errno, return it.
                    if let Some(err) = <$base as $crate::error::FromErrno>::try_from_errno(num) {
                        return Some(err.into());
                    }
                )?
                match num {
                    $($errno => Some($err),)*
                    // this function should only be called if errno is set
                    0 => panic!("PROS reported an error but none was found!"),
                    _ => None,
                }
            }
            #[allow(unused, clippy::redundant_closure_call)]
            fn unknown_variant(num: i32) -> Option<Self> {
                $(
                    if let Some(variant) = <$base as $crate::error::FromErrno>::unknown_variant(num) {
                        return Some(variant.into());
                    }
                )?
                #[allow(unused_mut)]
                let mut variant = None;
                $(
                    variant = Some($unknown(num));
                )?
                variant
            }
            fn from_errno(num: i32) -> Self {
                Self::try_from_errno(num)
                    .unwrap_or_else(|| Self::unknown_variant(num)
                        .unwrap_or_else(|| panic!("Unknown errno code {num}")))
            }
        }
    }
}
pub(crate) use map_errno;

/// If errno has an error, return early.
macro_rules! bail_errno {
    () => {{
        let errno = $crate::error::take_errno();
        if errno != 0 {
            let err = $crate::error::FromErrno::from_errno(errno);
            return Err(err);
        }
    }};
}
pub(crate) use bail_errno;

/// Checks if the value is equal to the error state, and if it is,
/// uses the value of errno to create an error and return early.
macro_rules! bail_on {
    ($err_state:expr, $val:expr) => {{
        let val = $val;
        #[allow(clippy::cmp_null)]
        if val == $err_state {
            let errno = $crate::error::take_errno();
            let err = $crate::error::FromErrno::from_errno(errno);
            return Err(err); // where are we using this in a function that doesn't return result?
        }
        val
    }};
}
pub(crate) use bail_on;
use snafu::Snafu;

pub trait FromErrno {
    /// Consume the current `errno` and, if it contains a known error, returns Self.
    fn try_from_errno(num: i32) -> Option<Self>
    where
        Self: Sized;
    /// The variant to return if the errno value is unknown.
    fn unknown_variant(num: i32) -> Option<Self>
    where
        Self: Sized;
    /// Consume the current `errno` and returns Self.
    /// If the error is unknown, returns the result of [`Self::unknown_variant`].
    fn from_errno(num: i32) -> Self
    where
        Self: Sized;
}

#[derive(Debug, Snafu)]
pub enum PortError {
    #[snafu(display("The port you specified is outside of the allowed range!"))]
    PortOutOfRange,
    #[snafu(display(
        // used to have "Is something else plugged in?" But the vex radio (link) uses the same errno, so that's not always applicable.
        "The port you specified couldn't be configured the requested smart device type."
    ))]
    PortCannotBeConfigured,
    #[snafu(display("{source}"), context(false))]
    Unknown { source: ErrnoError },
}

map_errno!(PortError {
    ENXIO => Self::PortOutOfRange,
    ENODEV => Self::PortCannotBeConfigured,
});

#[derive(Debug, Snafu)]
#[snafu(display("An unknown error occurred (errno {errno})."))]
pub struct ErrnoError {
    pub errno: i32,
}

map_errno! {
    ErrnoError {}
    unknown = |errno| Self { errno };
}
