use no_std_io::io;
use pros_sys::PROS_ERR;
use snafu::Snafu;

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::error::{bail_on, map_errno, PortError};

#[derive(Debug, Eq, PartialEq)]
pub struct SerialPort {
    port: SmartPort,
}

impl SerialPort {
    /// Open and configure a serial port on a [`SmartPort`].
    ///
    /// This configures a smart port to act as a generic serial device, capable of sending/recieving
    /// data.
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
            pros_sys::serial_write(self.port.index(), buf.as_ptr(), buf.len() as i32)
        }) as usize)
    }

    fn flush(&mut self) -> Result<(), SerialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_flush(self.port.index())
        });

        Ok(())
    }

    /// Read the next byte available in the port's input buffer.
    pub fn read_byte(&self) -> Result<Option<u8>, SerialError> {
        let read = bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_read_byte(self.port.index())
        });

        Ok(match read {
            -1 => None,
            _ => Some(read as u8),
        })
    }

    /// Read the next byte available in the port's input buffer without removing it.
    pub fn peek_byte(&self) -> Result<Option<u8>, SerialError> {
        let peeked = bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_peek_byte(self.port.index())
        });

        Ok(match peeked {
            -1 => None,
            _ => Some(peeked as u8),
        })
    }

    /// Write the single byte to the port's output buffer.
    pub fn write_byte(&mut self, byte: u8) -> Result<usize, SerialError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_write_byte(self.port.index(), byte)
        }) as usize)
    }

    // Returns the number of bytes available to be read in the the port's FIFO input buffer.
    pub fn bytes_to_read(&self) -> Result<usize, SerialError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_get_read_avail(self.port.index())
        }) as usize)
    }

    /// Returns the number of bytes free in the port's FIFO output buffer.
    pub fn available_write_bytes(&self) -> Result<usize, SerialError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::serial_get_write_free(self.port.index())
        }) as usize)
    }
}

impl io::Read for SerialPort {
    /// Read some bytes from this serial port into the specified buffer, returning
    /// how many bytes were read.
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
    /// Write a buffer into the serial port's output buffer, returning how many bytes were written.
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

    /// Flush the serial port's output buffer, ensuring that all intermediately buffered
    /// contents reach their destination.
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

#[derive(Debug, Snafu)]
pub enum SerialError {
    #[snafu(display("Serious internal write error occurred."))]
    InternalWriteError,
    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
}

map_errno! {
    SerialError {
        EIO => Self::InternalWriteError,
    }
    inherit PortError;
}
