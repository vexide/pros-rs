use core::time::Duration;

use log::{Level, Log, Metadata, Record, SetLoggerError};

struct ProsLogger;

impl Log for ProsLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
            let level_string = format!("{:<5}", record.level().to_string())

            let target = if !record.target().is_empty() {
                record.target()
            } else {
                record.module_path().unwrap_or_default()
            };

            let now = Duration::from unsafe { pros_sys::millis()};

            let message = format!(
                "{}{} [{}{}] {}",
                timestamp,
                level_string,
                target,
                thread,
                record.args()
            );

            #[cfg(not(feature = "stderr"))]
            println!("{}", message);

            #[cfg(feature = "stderr")]
            eprintln!("{}", message);
        }
    }

    fn flush(&self) {}
}

/// Configure the console to display colours.
///
/// This is only needed on Windows when using the 'colored' feature.
#[cfg(all(windows, feature = "colored"))]
pub fn set_up_color_terminal() {
    use std::io::{stdout, IsTerminal};

    if stdout().is_terminal() {
        unsafe {
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            use windows_sys::Win32::System::Console::{
                GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE,
                ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
            };

            let stdout = GetStdHandle(STD_OUTPUT_HANDLE);

            if stdout == INVALID_HANDLE_VALUE {
                return;
            }

            let mut mode: CONSOLE_MODE = 0;

            if GetConsoleMode(stdout, &mut mode) == 0 {
                return;
            }

            SetConsoleMode(stdout, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
    }
}

/// Configure the console to display colours.
///
/// This method does nothing if not running on Windows with the colored feature.
#[cfg(not(all(windows, feature = "colored")))]
pub fn set_up_color_terminal() {}

/// Initialise the logger with its default configuration.
///
/// Log messages will not be filtered.
/// The `RUST_LOG` environment variable is not used.
pub fn init() -> Result<(), SetLoggerError> {
    SimpleLogger::new().init()
}

/// Initialise the logger with its default configuration.
///
/// Log messages will not be filtered.
/// The `RUST_LOG` environment variable is not used.
///
/// This function is only available if the `timestamps` feature is enabled.
#[cfg(feature = "timestamps")]
pub fn init_utc() -> Result<(), SetLoggerError> {
    SimpleLogger::new().with_utc_timestamps().init()
}

/// Initialise the logger with the `RUST_LOG` environment variable.
///
/// Log messages will be filtered based on the `RUST_LOG` environment variable.
pub fn init_with_env() -> Result<(), SetLoggerError> {
    SimpleLogger::new().env().init()
}

/// Initialise the logger with a specific log level.
///
/// Log messages below the given [`Level`] will be filtered.
/// The `RUST_LOG` environment variable is not used.
pub fn init_with_level(level: Level) -> Result<(), SetLoggerError> {
    SimpleLogger::new()
        .with_level(level.to_level_filter())
        .init()
}
