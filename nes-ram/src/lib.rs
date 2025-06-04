use nes_base::{RAM, Reader, Writer};

pub struct RAMImpl {
    data: Vec<u8>,
}

impl RAMImpl {
    pub fn new(size: usize) -> Self {
        RAMImpl {
            data: vec![0; size],
        }
    }
}

impl Reader for RAMImpl {
    fn read(&self, addr: u16) -> u8 {
        if (addr as usize) < self.data.len() {
            self.data[addr as usize]
        } else {
            panic!("Read out of bounds: {}", addr);
        }
    }
}

impl Writer for RAMImpl {
    fn write(&mut self, addr: u16, data: u8) {
        if (addr as usize) < self.data.len() {
            self.data[addr as usize] = data;
        } else {
            panic!("Write out of bounds: {}", addr);
        }
    }
}

impl RAM for RAMImpl {}
