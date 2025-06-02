use std::{cell::RefCell, rc::Rc};

pub trait Reader {
    fn read(&self, addr: u16) -> u8;
    fn read_u16(&self, addr: u16) -> u16 {
        let low = self.read(addr) as u16;
        let high = self.read(addr + 1) as u16;
        (high << 8) | low
    }
}
pub trait Writer {
    fn write(&mut self, addr: u16, data: u8);
    fn write_u16(&mut self, addr: u16, data: u16) {
        let low = (data & 0xFF) as u8;
        let high = (data >> 8) as u8;
        self.write(addr, low);
        self.write(addr + 1, high);
    }
}

// 连接设备和总线的适配器
pub trait BusAdapter: Reader + Writer {
    fn address_accept(&self, addr: u16) -> bool;
}

pub trait Bus: Reader + Writer {
    fn register_device(&mut self, device: Rc<RefCell<dyn BusAdapter>>);
}
