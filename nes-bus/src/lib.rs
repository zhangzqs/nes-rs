use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use nes_base::{Bus, BusAdapter, Reader, Writer};

pub struct BusImpl {
    devices: Vec<Rc<RefCell<dyn BusAdapter>>>,
    open_bus: Cell<u8>,
}

impl Default for BusImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl BusImpl {
    pub fn new() -> Self {
        BusImpl {
            devices: Default::default(),
            open_bus: Cell::new(0),
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
                let value = device.borrow().read(address);
                self.open_bus.set(value);
                return value;
            }
        }
        self.open_bus.get()
    }
}

impl Writer for BusImpl {
    fn write(&mut self, address: u16, data: u8) {
        self.open_bus.set(data);
        for device in &mut self.devices {
            if device.borrow().address_accept(address) {
                device.borrow_mut().write(address, data);
                return;
            }
        }
    }
}

impl BusAdapter for BusImpl {
    fn observe_ppu_address(&self, addr: u16, cycle: u64) {
        for device in &self.devices {
            device.borrow().observe_ppu_address(addr, cycle);
        }
    }
    fn take_dma_request(&self) -> Option<u8> {
        self.devices
            .iter()
            .find_map(|device| device.borrow().take_dma_request())
    }
    fn address_accept(&self, addr: u16) -> bool {
        self.devices
            .iter()
            .any(|device| device.borrow().address_accept(addr))
    }
}
