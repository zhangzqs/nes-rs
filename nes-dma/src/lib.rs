use std::{cell::RefCell, rc::Rc};

use nes_base::{DMA, Reader, Writer};

pub struct DMAImpl {
    source: Rc<RefCell<dyn Reader>>,
    dest: Rc<RefCell<dyn Writer>>,
}

impl DMA for DMAImpl {
    fn transfer(&mut self, source_addr: u16, dest_addr: u16, length: usize) {
        if length > 256 {
            panic!("DMA transfer length cannot exceed 256 bytes");
        }

        for i in 0..length {
            let data = self.source.borrow().read(source_addr + i as u16);
            self.dest.borrow_mut().write(dest_addr + i as u16, data);
        }
    }
}
