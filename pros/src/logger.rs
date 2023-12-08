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
        let target = if !record.target().is_empty() {
            record.target()
        } else {
            record.module_path().unwrap_or_default()
        };

        let now =
            chrono::Duration::from_std(Duration::from_millis(unsafe { pros_sys::millis() as _ }))
                .unwrap();

        let message = format!(
            "{}m{}s{}ms [{}] {}: {}",
            now.num_minutes(),
            now.num_seconds(),
            now.num_milliseconds(),
            record.level(),
            target,
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
