use std::{cell::RefCell, rc::Rc};

use crate::{BusAdapter, Reader, Writer};

pub trait Cartridge {
    fn cpu_read(&self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, value: u8);
    fn ppu_read(&self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, value: u8);
    fn mirroring(&self) -> Mirroring;
}

#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreen,
    FourScreen,
}

pub struct CartridgeAdapterForCPUBus(pub Rc<RefCell<dyn Cartridge>>);

impl Reader for CartridgeAdapterForCPUBus {
    fn read(&self, addr: u16) -> u8 {
        self.0.borrow().cpu_read(addr)
    }
}

impl Writer for CartridgeAdapterForCPUBus {
    fn write(&mut self, addr: u16, data: u8) {
        self.0.borrow_mut().cpu_write(addr, data);
    }
}

impl BusAdapter for CartridgeAdapterForCPUBus {
    fn address_accept(&self, addr: u16) -> bool {
        addr >= 0x4020
    }
}
