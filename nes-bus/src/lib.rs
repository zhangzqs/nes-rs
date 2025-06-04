use std::{cell::RefCell, rc::Rc};

use nes_base::{Bus, BusAdapter, Reader, Writer};

pub struct BusImpl {
    devices: Vec<Rc<RefCell<dyn BusAdapter>>>,
}

impl BusImpl {
    pub fn new() -> Self {
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
        panic!("Address out of range: 0x{:04X}", address);
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
        panic!("Address out of range: 0x{:04X}", address);
    }
}

impl BusAdapter for BusImpl {
    fn address_accept(&self, addr: u16) -> bool {
        self.devices
            .iter()
            .any(|device| device.borrow().address_accept(addr))
    }
}
