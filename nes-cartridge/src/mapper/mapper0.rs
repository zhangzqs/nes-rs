use crate::cartridge::ICartridge;
use crate::mapper::Mapper;
pub struct Mapper0<'a> {
    cartridge: &'a mut dyn ICartridge,
}

impl<'a> Mapper0<'a> {
    pub fn new(cartridge: &'a mut dyn ICartridge) -> Self {
        Self { cartridge }
    }
}

impl<'a> Mapper for Mapper0<'a> {
    fn cpu_map_read(&mut self, address: u16) -> u8 {
        // NROM mapping:
        // If PRG-ROM is 16KB:
        //   0x8000-0xBFFF: Map to 0x0000-0x3FFF
        //   0xC000-0xFFFF: Mirror 0x0000-0x3FFF
        // If PRG-ROM is 32KB:
        //   0x8000-0xFFFF: Map to 0x0000-0x7FFF

        if (0x8000..0xC000).contains(&address) {
            return self.cartridge.prg_rom()[(address - 0x8000) as usize];
        }

        let prg_banks = self.cartridge.prg_banks();
        if address >= 0xC000 {
            if prg_banks == 1 {
                return self.cartridge.prg_rom()[(address - 0xC000) as usize];
            }
            return self.cartridge.prg_rom()[(address - 0x8000) as usize];
        }

        panic!("Mapper0 cannot read by CPU at address 0x{:04X}", address);
    }

    fn cpu_map_write(&mut self, address: u16, value: u8) {
        // Normally NROM doesn't support writes to PRG-ROM, but we'll implement
        // it the same way as reads for completeness

        if (0x8000..0xC000).contains(&address) {
            // This would normally be illegal - NROM uses ROM which isn't writable
            // In a real emulator, you might want to log this as a warning
            let prg_rom = self.cartridge.prg_rom_mut();
            prg_rom[(address - 0x8000) as usize] = value;
            return;
        }

        let prg_banks = self.cartridge.prg_banks();
        if address >= 0xC000 {
            if prg_banks == 1 {
                let prg_rom = self.cartridge.prg_rom_mut();
                prg_rom[(address - 0xC000) as usize] = value;
                return;
            }
            let prg_rom = self.cartridge.prg_rom_mut();
            prg_rom[(address - 0x8000) as usize] = value;
            return;
        }
        panic!("Mapper0 cannot write by CPU at address 0x{:04X}", address);
    }

    fn ppu_map_read(&mut self, address: u16) -> u8 {
        if (0x0000..0x2000).contains(&address) {
            return self.cartridge.chr_rom()[address as usize];
        }
    }

    fn ppu_map_write(&mut self, address: u16, value: u8) {
        if (0x0000..0x2000).contains(&address) {
            if self.cartridge.chr_banks() == 0 {
                // Treat as RAM
                self.cartridge.chr_rom_mut()[address as usize] = value;
            }
            // Else: Normally CHR-ROM isn't writable
        }
    }
}
