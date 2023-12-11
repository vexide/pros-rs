use core::time::Duration;

use alloc::{ffi::CString, format};

use log::{Log, Metadata, Record};

#[derive(Default)]
pub struct ProsLogger;

impl ProsLogger {
    pub fn init() -> Result<(), log::SetLoggerError> {
        log::set_logger(&ProsLogger)?;
        log::set_max_level(log::LevelFilter::Trace);

        unsafe {
            pros_sys::lcd_initialize();
        }

        Ok(())
    }
}

impl Log for ProsLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let now =
            chrono::Duration::from_std(Duration::from_millis(unsafe { pros_sys::millis() as _ }))
                .unwrap();

        let time = if now.num_minutes() > 0 {
            format!("{}m{}s", now.num_minutes(), now.num_seconds() % 60)
        } else {
            format!("{}s", now.num_seconds())
        };

        let level = match record.level() {
            log::Level::Error => "E",
            log::Level::Warn => "W",
            log::Level::Info => "I",
            log::Level::Debug => "D",
            log::Level::Trace => "T",
        };

        let message = format!(
            "{time} {}: {}",
            level,
            record.args()
        );

        println!("{}", message);
        // Print to the debug teminal
        let c_output = CString::new(message).unwrap();
        unsafe {
            pros_sys::puts(c_output.as_ptr() as _);
        }
    }

    fn flush(&self) {}
}
