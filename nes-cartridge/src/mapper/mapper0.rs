use std::{cell::RefCell, rc::Rc};

use nes_base::Ram;

use crate::mapper::Mapper;

pub struct Mapper0 {
    prg_banks: u8,
    chr_rom: Rc<RefCell<Vec<u8>>>,
    prg_rom: Rc<RefCell<Vec<u8>>>,
    sram: Option<Rc<RefCell<dyn Ram>>>,
}

impl Mapper0 {
    pub fn new(
        prg_banks: u8,
        chr_rom: Rc<RefCell<Vec<u8>>>,
        prg_rom: Rc<RefCell<Vec<u8>>>,
        sram: Option<Rc<RefCell<dyn Ram>>>,
    ) -> Self {
        Mapper0 {
            prg_banks,
            chr_rom,
            prg_rom,
            sram,
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..0x2000 => {
                // CHR ROM
                let chr_rom = self.chr_rom.borrow();
                chr_rom[addr as usize]
            }
            0x6000..0x8000 => {
                // SRAM
                if let Some(ref sram) = self.sram {
                    sram.borrow().read(addr - 0x6000)
                } else {
                    panic!("SRAM not available");
                }
            }
            0x8000..0xC000 => {
                // PRG ROM Bank 1
                let prg_rom = self.prg_rom.borrow();
                prg_rom[addr as usize - 0x8000]
            }
            0xC000..=0xFFFF => {
                // PRG ROM Bank 2
                let prg_rom = self.prg_rom.borrow();
                if self.prg_banks == 1 {
                    prg_rom[addr as usize - 0xC000]
                } else {
                    prg_rom[addr as usize - 0x8000]
                }
            }
            _ => panic!("Invalid address: 0x{:04X}", addr),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..0x2000 => {
                // CHR ROM is read-only in Mapper 0
                panic!("Attempt to write to CHR ROM at address 0x{:04X}", addr);
            }
            0x6000..0x8000 => {
                // SRAM
                if let Some(ref mut sram) = self.sram {
                    sram.borrow_mut().write(addr - 0x6000, value);
                } else {
                    panic!("SRAM not available");
                }
            }
            0x8000..0xC000 => {
                // PRG ROM Bank 1
                let mut prg_rom = self.prg_rom.borrow_mut();
                prg_rom[addr as usize - 0x8000] = value;
            }
            0xC000..=0xFFFF => {
                // PRG ROM Bank 2
                let mut prg_rom = self.prg_rom.borrow_mut();
                if self.prg_banks == 1 {
                    prg_rom[addr as usize - 0xC000] = value;
                } else {
                    prg_rom[addr as usize - 0x8000] = value;
                }
            }
            _ => panic!("Invalid address: 0x{:04X}", addr),
        }
    }
}

impl Mapper for Mapper0 {
    fn cpu_read(&self, addr: u16) -> u8 {
        self.read(addr)
    }

    fn cpu_write(&mut self, addr: u16, value: u8) {
        self.write(addr, value);
    }

    fn ppu_read(&self, addr: u16) -> u8 {
        self.read(addr)
    }

    fn ppu_write(&mut self, addr: u16, value: u8) {
        self.write(addr, value);
    }
}
