use nes_base::{Ram, Reader, Writer};

pub struct RamImpl {
    data: Vec<u8>,
}

impl RamImpl {
    pub fn new(size: usize) -> Self {
        RamImpl {
            data: vec![0; size],
        }
    }
}

impl Reader for RamImpl {
    fn read(&self, addr: u16) -> u8 {
        if (addr as usize) < self.data.len() {
            self.data[addr as usize]
        } else {
            panic!("Read out of bounds: {}", addr);
        }
    }
}

impl Writer for RamImpl {
    fn write(&mut self, addr: u16, data: u8) {
        if (addr as usize) < self.data.len() {
            self.data[addr as usize] = data;
        } else {
            panic!("Write out of bounds: {}", addr);
        }
    }
}

impl Ram for RamImpl {
    fn reset(&mut self) {
        for byte in &mut self.data {
            *byte = 0;
        }
    }
}
