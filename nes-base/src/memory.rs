use std::{cell::RefCell, rc::Rc};

use crate::{BusAdapter, Reader, Writer};

pub trait Ram: Reader + Writer {
    fn reset(&mut self);
}

pub struct RamAdapterForCpuBus(pub Rc<RefCell<dyn Ram>>);

impl Reader for RamAdapterForCpuBus {
    fn read(&self, addr: u16) -> u8 {
        self.0.borrow().read(addr % 0x800)
    }
}

impl Writer for RamAdapterForCpuBus {
    fn write(&mut self, addr: u16, data: u8) {
        self.0.borrow_mut().write(addr % 0x800, data);
    }
}

impl BusAdapter for RamAdapterForCpuBus {
    fn address_accept(&self, addr: u16) -> bool {
        addr < 0x2000 // 2KB of RAM
    }
}
