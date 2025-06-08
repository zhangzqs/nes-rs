use std::{cell::RefCell, rc::Rc};

use crate::{BusAdapter, Reader, Writer};

pub trait RAM: Reader + Writer {
    fn reset(&mut self);
}

pub struct RAMAdapterForCPUBus(pub Rc<RefCell<dyn RAM>>);

impl Reader for RAMAdapterForCPUBus {
    fn read(&self, addr: u16) -> u8 {
        self.0.borrow().read(addr % 0x800)
    }
}

impl Writer for RAMAdapterForCPUBus {
    fn write(&mut self, addr: u16, data: u8) {
        self.0.borrow_mut().write(addr % 0x800, data);
    }
}

impl BusAdapter for RAMAdapterForCPUBus {
    fn address_accept(&self, addr: u16) -> bool {
        addr < 0x2000 // 2KB of RAM
    }
}
