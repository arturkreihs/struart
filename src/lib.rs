use serial2::SerialPort;
use thiserror::Error;
use std::sync::RwLock;

#[derive(Error, Debug)]
pub enum StruartError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("RwLock was Poisoned")]
    RwLockPoisoned,
}

pub struct Struart {
    port: SerialPort,
    buffer: RwLock<Vec<u8>>,
}

impl Default for Struart {
    fn default() -> Self {
        Self::new("/dev/ttyUSB0", 115200).unwrap()
    }
}

impl Struart {
    pub fn new(dev: &str, speed: u32) -> Result<Self, StruartError> {
        let mut port = SerialPort::open(dev, speed)?;
        port.set_read_timeout(core::time::Duration::from_millis(100))?;
        port.set_write_timeout(core::time::Duration::from_millis(1000))?;

        Ok(Self {
            port,
            buffer: RwLock::new(vec![]),
        })
    }

    pub fn send(&self, text: &str) -> Result<(), StruartError> {
        self.port.write(text.as_bytes())?;
        self.port.write(&[13u8, 10u8])?;
        self.port.flush()?;
        Ok(())
    }

    pub fn read<F: Fn(&str)>(&self, cb: F) -> Result<(), StruartError> {
        let mut buf = [0u8; 256];
        while let Ok(len) = self.port.read(&mut buf) {
            for byte in buf.iter().take(len) {
                match byte {
                    0x0d => { // newline char
                        cb(&String::from_utf8_lossy(&self.buffer.read().map_err(|_| StruartError::RwLockPoisoned)?));
                        self.buffer.write().map_err(|_| StruartError::RwLockPoisoned)?.clear();
                    },
                    0x20..=0x7e => self.buffer.write().map_err(|_| StruartError::RwLockPoisoned)?.push(*byte), // printable char
                    _ => continue,
                }
            }
        }
        Ok(())
    }
}
