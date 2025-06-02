use std::{cell::RefCell, rc::Rc};

use nes_base::{Cartridge, Memory};
use nes_file::NESFile;
use nes_ram::RAMImpl;

mod mapper;
mod nes_file;

pub struct CartridgeImpl {
    nes: NESFile,
    sram: Option<Rc<RefCell<dyn Memory>>>,
}

impl CartridgeImpl {
    pub fn new(nes: NESFile) -> Self {
        let has_battery_backed = nes.has_battery_backed();
        CartridgeImpl {
            nes,
            sram: if has_battery_backed {
                Some(Rc::new(RefCell::new(RAMImpl::new(0x2000))))
            } else {
                None
            },
        }
    }
}

impl Cartridge for CartridgeImpl {
    fn cpu_read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {
        todo!()
    }

    fn ppu_read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        todo!()
    }
}