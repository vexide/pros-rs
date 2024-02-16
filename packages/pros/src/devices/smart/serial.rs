//! Generic serial device module.
//!
//! Provides support for using [`SmartPort`]s as generic serial communication devices.

use no_std_io::io;
use pros_sys::PROS_ERR;
use snafu::Snafu;

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::error::{bail_on, map_errno, PortError};

/// Represents a smart port configured as a generic serial controller.
#[derive(Debug, Eq, PartialEq)]
pub struct SerialPort {
    port: SmartPort,
}

impl SerialPort {
    /// Open and configure a serial port on a [`SmartPort`].
    ///
    /// This configures a [`SmartPort`] to act as a generic serial controller capable of sending/recieving
    /// data. Providing a baud rate, or the transmission rate of bits is required. The maximum theoretical
    /// baud rate is 921600.
    ///
    /// # Examples
    ///
    /// ```
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    /// ```
    pub fn open(port: SmartPort, baud_rate: u32) -> Result<Self, SerialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::serial_enable(port.index()));
            bail_on!(
                PROS_ERR,
                // libv5rt allows passing in negative baudrate for internal reasons. Other than
                // for very specific cases, this at best isn't useful and at worst is undefined
                // behavior, so we take a u32 baudrate and call it a day.
                pros_sys::serial_set_baudrate(port.index(), baud_rate as i32)
            );
        }

        Ok(Self { port })
    }

    fn recieve(&self, buf: &mut [u8]) -> Result<usize, SerialError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_read(self.port.index(), buf.as_mut_ptr(), buf.len() as i32)
        }) as usize)
    }

    fn transmit(&mut self, buf: &[u8]) -> Result<usize, SerialError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_write(self.port.index(), buf.as_ptr() as *mut u8, buf.len() as i32)
        }) as usize)
    }

    fn flush(&mut self) -> Result<(), SerialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_flush(self.port.index())
        });

        Ok(())
    }

    /// Read the next byte available in the serial port's input buffer, or `None` if the input
    /// buffer is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// loop {
    ///     if let Some(byte) = serial.read_byte()? {
    ///         println!("Got byte: {}", byte);
    ///     }
    ///     pros::task::delay(Duration::from_millis(10));
    /// }
    /// ```
    pub fn read_byte(&self) -> Result<Option<u8>, SerialError> {
        let read = bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_read_byte(self.port.index())
        });

        Ok(match read {
            -1 => None,
            _ => Some(read as u8),
        })
    }

    /// Read the next byte available in the port's input buffer without removing it. Returns
    /// `None` if the input buffer is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// if let Some(next_byte) = serial.peek_byte()? {
    ///     println!("Next byte: {}", next_byte);
    /// }
    /// ```
    pub fn peek_byte(&self) -> Result<Option<u8>, SerialError> {
        let peeked = bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_peek_byte(self.port.index())
        });

        Ok(match peeked {
            -1 => None,
            _ => Some(peeked as u8),
        })
    }

    /// Write a single byte to the port's output buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// // Write 0x80 (128u8) to the output buffer
    /// serial.write_byte(0x80)?;
    /// ```
    pub fn write_byte(&mut self, byte: u8) -> Result<usize, SerialError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_write_byte(self.port.index(), byte)
        }) as usize)
    }

    /// Returns the number of bytes available to be read in the the port's FIFO input buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// if serial.byets_to_read()? > 0 {
    ///     println!("{}", serial.read_byte()?.unwrap());
    /// }
    /// ```
    pub fn bytes_to_read(&self) -> Result<usize, SerialError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_get_read_avail(self.port.index())
        }) as usize)
    }

    /// Returns the number of bytes free in the port's FIFO output buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// if serial.available_write_bytes()? > 0 {
    ///     serial.write_byte(0x80)?;
    /// }
    /// ```
    pub fn available_write_bytes(&self) -> Result<usize, SerialError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_get_write_free(self.port.index())
        }) as usize)
    }
}

impl io::Read for SerialPort {
    /// Read some bytes from this serial port into the specified buffer, returning
    /// how many bytes were read.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// let mut buffer = Vec::new();
    ///
    /// loop {
    ///     serial.read(&mut buffer);
    ///     pros::task::delay(Duration::from_millis(10));
    /// }
    /// ```
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.recieve(buf).map_err(|err| match err {
            SerialError::InternalWriteError => io::ErrorKind::Other,
            SerialError::Port { source } => match source {
                PortError::PortOutOfRange => io::ErrorKind::AddrNotAvailable,
                PortError::PortCannotBeConfigured => io::ErrorKind::AddrInUse,
            },
        })?;

        Ok(bytes_read)
    }
}

impl io::Write for SerialPort {
    /// Write a buffer into the serial port's output buffer, returning how many bytes
    /// were written.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// buffer.write(b"some bytes")?;
    /// ```
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes_written = self.transmit(buf).map_err(|err| match err {
            SerialError::InternalWriteError => io::ErrorKind::Other,
            SerialError::Port { source } => match source {
                PortError::PortOutOfRange => io::ErrorKind::AddrNotAvailable,
                PortError::PortCannotBeConfigured => io::ErrorKind::AddrInUse,
            },
        })?;

        Ok(bytes_written)
    }

    /// Clears the internal input and output FIFO buffers.
    ///
    /// This can be useful to reset state and remove old, potentially unneeded data
    /// from the input FIFO buffer or to cancel sending any data in the output FIFO
    /// buffer.
    ///
    /// # Flushing does not send data.
    ///
    /// This function does not cause the data in the output buffer to be
    /// written, it simply clears the internal buffers. Unlike stdout, generic
    /// serial does not use buffered IO (the FIFO buffers are written as soon
    /// as possible).
    ///
    /// ```
    /// let mut serial = SerialPort::open(peripherals.port_1, 115200)?;
    ///
    /// buffer.write(b"some bytes")?;
    /// buffer.flush()?;
    /// ```
    fn flush(&mut self) -> io::Result<()> {
        Ok(self.flush().map_err(|err| match err {
            SerialError::InternalWriteError => io::ErrorKind::Other,
            SerialError::Port { source } => match source {
                PortError::PortOutOfRange => io::ErrorKind::AddrNotAvailable,
                PortError::PortCannotBeConfigured => io::ErrorKind::AddrInUse,
            },
        })?)
    }
}

impl SmartDevice for SerialPort {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Serial
    }
}

/// Errors that can occur when interacting with a [`SerialPort`].
#[derive(Debug, Snafu)]
pub enum SerialError {
    /// Serious internal write error occurred.
    InternalWriteError,

    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}

map_errno! {
    SerialError {
        EIO => Self::InternalWriteError,
    }
    inherit PortError;
}
