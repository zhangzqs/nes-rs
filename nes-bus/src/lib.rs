use std::{cell::RefCell, rc::Rc};

use nes_base::{Bus, BusAdapter, Reader, Writer};

struct BusImpl {
    devices: Vec<Rc<RefCell<dyn BusAdapter>>>,
}

impl BusImpl {
    fn new() -> Self {
        BusImpl {
            devices: Default::default(),
        }
    }
}

impl Bus for BusImpl {
    fn register_device(&mut self, device: Rc<RefCell<dyn BusAdapter>>) {
        self.devices.push(device);
    }
}

impl Reader for BusImpl {
    fn read(&self, address: u16) -> u8 {
        for device in &self.devices {
            if device.borrow().address_accept(address) {
                return device.borrow().read(address);
            }
        }
        panic!("Address out of range: {}", address);
    }
}

impl Writer for BusImpl {
    fn write(&mut self, address: u16, data: u8) {
        for device in &mut self.devices {
            if device.borrow().address_accept(address) {
                device.borrow_mut().write(address, data);
                return;
            }
        }
        panic!("Address out of range: {}", address);
    }
}
