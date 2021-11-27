use std::fs::File;
use std::io::Read;

pub struct ROM {
    pub rom: [u8; 3584],
    pub size: usize,
}

impl ROM {
    pub fn from(filename: &str) -> Self {
        let mut f = File::open(filename).expect("File not found");
        let mut buffer = [0 as u8; 3584];

        let bytes_read = if let Ok(bytes_read) = f.read(&mut buffer) {
            bytes_read
        } else {
            0
        };

        ROM {
            rom: buffer,
            size: bytes_read,
        }
    }
}