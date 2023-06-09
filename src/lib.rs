use serial2::SerialPort;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StruartError {
    #[error("IO")]
    IO(#[from] std::io::Error),
}

pub struct Struart {
    port: SerialPort,
    buf: Vec<u8>,
}

impl Default for Struart {
    fn default() -> Self {
        Self::new("/dev/ttyUSB0", 115200)
    }
}

impl Struart {
    pub fn new(dev: &str, speed: u32) -> Self {
        let mut port = SerialPort::open(dev, speed).unwrap();
        port.set_read_timeout(core::time::Duration::from_millis(1000)).unwrap();
        port.set_write_timeout(core::time::Duration::from_millis(1000)).unwrap();

        Self {
            port,
            buf: vec![],
        }
    }

    pub fn send(&self, text: &str) -> Result<(), StruartError> {
        self.port.write(text.as_bytes())?;
        self.port.write(&[13u8, 10u8])?;
        self.port.flush()?;
        Ok(())
    }

    pub fn read<F: Fn(&str)>(&mut self, cb: F) -> Result<(), StruartError> {
        let mut buf = [0u8; 256];
        while let Ok(len) = self.port.read(&mut buf) {
            for byte in buf.iter().take(len) {
                match byte {
                    0x0d => { // newline char
                        cb(&String::from_utf8_lossy(&self.buf));
                        self.buf.clear();
                    },
                    0x20..=0x7e => self.buf.push(*byte), // printable char
                    _ => continue,
                }
            }
        }
        Ok(())
    }
}
