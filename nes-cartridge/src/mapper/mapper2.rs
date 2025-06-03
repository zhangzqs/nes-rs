use std::{cell::RefCell, rc::Rc};

use nes_base::{Cartridge, Memory};

pub struct Mapper2 {
    prg_banks: u8,
    prg_bank1: u8,
    prg_bank2: u8,
    chr_rom: Rc<RefCell<Vec<u8>>>,
    prg_rom: Rc<RefCell<Vec<u8>>>,
    sram: Option<Rc<RefCell<dyn Memory>>>,
}

impl Mapper2 {
    pub fn new(
        prg_banks: u8,
        chr_rom: Rc<RefCell<Vec<u8>>>,
        prg_rom: Rc<RefCell<Vec<u8>>>,
        sram: Option<Rc<RefCell<dyn Memory>>>,
    ) -> Self {
        let prg_bank1 = 0;
        let prg_bank2 = prg_banks - 1;
        Mapper2 {
            prg_banks,
            prg_bank1,
            prg_bank2,
            chr_rom,
            prg_rom,
            sram,
        }
    }
}

impl Mapper2 {
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
                prg_rom[(self.prg_bank1 as usize * 0x4000 + (addr as usize - 0xC000)) as usize]
            }
            0xC000.. => {
                // PRG ROM Bank 2
                let prg_rom = self.prg_rom.borrow();
                prg_rom[(self.prg_bank2 as usize * 0x4000 + (addr as usize - 0x8000)) as usize]
            }
            _ => panic!("Address out of range: {}", addr),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..0x2000 => {
                // CHR ROM is read-only, no write operation
                panic!("Cannot write to CHR ROM at address: {}", addr);
            }
            0x6000..0x8000 => {
                // SRAM
                if let Some(ref mut sram) = self.sram {
                    sram.borrow_mut().write(addr - 0x6000, value);
                } else {
                    panic!("SRAM not available");
                }
            }
            0x8000.. => {
                // PRG ROM Bank selection
                self.prg_bank1 = value % self.prg_banks;
            }
            _ => panic!("Write out of range: {}", addr),
        }
    }
}

impl Cartridge for Mapper2 {
    fn cpu_read(&self, addr: u16) -> u8 {
        self.read(addr)
    }

    fn cpu_write(&mut self, addr: u16, value: u8) {
        self.write(addr, value);
    }

    fn ppu_read(&self, addr: u16) -> u8 {
        if addr < 0x2000 {
            // CHR ROM
            let chr_rom = self.chr_rom.borrow();
            chr_rom[addr as usize]
        } else {
            panic!("PPU read out of range: {}", addr);
        }
    }

    fn ppu_write(&mut self, addr: u16, _: u8) {
        if addr < 0x2000 {
            // CHR ROM is read-only, no write operation
            panic!("Cannot write to CHR ROM at address: {}", addr);
        } else {
            panic!("PPU write out of range: {}", addr);
        }
    }
}
