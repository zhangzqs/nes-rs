use std::{cell::RefCell, rc::Rc};

use nes_base::Cartridge;

pub struct Mapper2 {
    prg_banks: u8,
    prg_bank1: u8,
    prg_bank2: u8,
    chr_rom: Rc<RefCell<Vec<u8>>>,
    prg_rom: Rc<RefCell<Vec<u8>>>,
    sram: Option<Rc<RefCell<Vec<u8>>>>,
}

impl Mapper2 {
    pub fn new(
        prg_banks: u8,
        chr_rom: Vec<u8>,
        prg_rom: Vec<u8>,
        sram: Option<Rc<RefCell<Vec<u8>>>>,
    ) -> Self {
        let prg_bank1 = 0;
        let prg_bank2 = prg_banks - 1;
        Mapper2 {
            prg_banks,
            prg_bank1,
            prg_bank2,
            chr_rom: Rc::new(RefCell::new(chr_rom)),
            prg_rom: Rc::new(RefCell::new(prg_rom)),
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
                    let sram_borrowed = sram.borrow();
                    sram_borrowed[(addr as usize - 0x6000) % sram_borrowed.len()]
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

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..0x2000 => {
                // CHR ROM is read-only, no write operation
                panic!("Cannot write to CHR ROM at address: {}", addr);
            }
            0x6000..0x8000 => {
                // SRAM
                if let Some(ref mut sram) = self.sram {
                    let mut sram_borrowed = sram.borrow_mut();
                    let sram_size = sram_borrowed.len();
                    sram_borrowed[(addr as usize - 0x6000) % sram_size] = data;
                } else {
                    panic!("SRAM not available");
                }
            }
            0x8000.. => {
                // PRG ROM Bank selection
                self.prg_bank1 = data % self.prg_banks;
            }
            _ => panic!("Write out of range: {}", addr),
        }
    }
}

impl Cartridge for Mapper2 {
    fn cpu_read(&self, addr: u16) -> u8 {
        self.read(addr)
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {
        self.write(addr, data);
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

    fn ppu_write(&mut self, addr: u16, data: u8) {
        if addr < 0x2000 {
            // CHR ROM is read-only, no write operation
            panic!("Cannot write to CHR ROM at address: {}", addr);
        } else {
            panic!("PPU write out of range: {}", addr);
        }
    }
}
