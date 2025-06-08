use nes_base::{Cartridge, RAM};
use nes_ram::RAMImpl;
use std::{cell::RefCell, rc::Rc};

mod mapper;
mod nes_file;

pub use nes_file::NESFile;

pub struct CartridgeImpl {
    mapper: Box<dyn Cartridge>,
}

impl CartridgeImpl {
    pub fn new(nes: NESFile) -> Self {
        let mapper_id = nes.header().mapper_id;
        let prg_banks = nes.header().prg_banks;
        let has_battery_backed = nes.header().has_battery_backed;
        let chr_rom = Rc::new(RefCell::new(nes.chr_rom()));
        let prg_rom = Rc::new(RefCell::new(nes.prg_rom()));
        let sram: Option<Rc<RefCell<dyn RAM>>> = if has_battery_backed {
            Some(Rc::new(RefCell::new(RAMImpl::new(0x2000))))
        } else {
            None
        };
        CartridgeImpl {
            mapper: mapper::get_mapper_by_id(mapper_id, prg_banks, chr_rom, prg_rom, sram),
        }
    }
}

impl Cartridge for CartridgeImpl {
    fn cpu_read(&self, addr: u16) -> u8 {
        self.mapper.cpu_read(addr)
    }

    fn cpu_write(&mut self, addr: u16, value: u8) {
        self.mapper.cpu_write(addr, value);
    }

    fn ppu_read(&self, addr: u16) -> u8 {
        self.mapper.ppu_read(addr)
    }

    fn ppu_write(&mut self, addr: u16, value: u8) {
        self.mapper.ppu_write(addr, value);
    }
}
