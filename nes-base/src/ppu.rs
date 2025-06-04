use std::{cell::RefCell, rc::Rc};

use crate::{BusAdapter, Reader, Writer};

pub trait PPU {
    fn write_reg_control(&mut self, value: u8);

    fn write_reg_mask(&mut self, value: u8);

    fn read_reg_status(&self) -> u8;

    fn write_reg_oam_addr(&mut self, value: u8);

    fn read_reg_oam_data(&self) -> u8;
    fn write_reg_oam_data(&mut self, value: u8);

    fn write_reg_scroll(&mut self, value: u8);

    fn write_reg_address(&mut self, value: u8);

    fn read_reg_data(&self) -> u8;
    fn write_reg_data(&mut self, value: u8);

    fn reset(&mut self);
    fn clock(&mut self);
    fn attach_bus(&mut self, bus: Rc<RefCell<dyn BusAdapter>>);
}

pub struct PPUBusAdapter(pub Rc<RefCell<dyn PPU>>);

impl Reader for PPUBusAdapter {
    fn read(&self, addr: u16) -> u8 {
        match (addr - 0x2000) % 8 {
            2 => self.0.borrow().read_reg_status(),
            4 => self.0.borrow().read_reg_oam_data(),
            7 => self.0.borrow().read_reg_data(),
            _ => panic!("PPU read from unsupported address: {:#04X}", addr),
        }
    }
}

impl Writer for PPUBusAdapter {
    fn write(&mut self, addr: u16, data: u8) {
        match (addr - 0x2000) % 8 {
            0 => self.0.borrow_mut().write_reg_control(data),
            1 => self.0.borrow_mut().write_reg_mask(data),
            3 => self.0.borrow_mut().write_reg_oam_addr(data),
            4 => self.0.borrow_mut().write_reg_oam_data(data),
            5 => self.0.borrow_mut().write_reg_scroll(data),
            6 => self.0.borrow_mut().write_reg_address(data),
            7 => self.0.borrow_mut().write_reg_data(data),
            _ => panic!("PPU write to unsupported address: {:#04X}", addr),
        }
    }
}

impl BusAdapter for PPUBusAdapter {
    fn address_accept(&self, addr: u16) -> bool {
        return addr >= 0x2000 && addr < 0x4000;
    }
}
