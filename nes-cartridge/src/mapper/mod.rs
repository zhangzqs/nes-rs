use nes_base::Mirroring;
mod mapper0;
mod mapper1;
mod mapper2;
mod mapper3;
mod mapper4;
mod mapper66;
mod mapper7;

/// Mappers translate addresses and own registers; cartridge memory is stored once by CartridgeImpl.
pub trait Mapper {
    fn prg_address(&self, addr: u16, size: usize) -> usize;
    fn chr_address(&self, addr: u16) -> usize {
        addr as usize
    }
    fn write_register(&mut self, _addr: u16, _value: u8, _cpu_cycle: u64) {}
    fn mirroring(&self) -> Option<Mirroring> {
        None
    }
    fn ram_enabled(&self) -> bool {
        true
    }
    fn ram_writable(&self) -> bool {
        self.ram_enabled()
    }
    fn observe_ppu_address(&mut self, _addr: u16, _ppu_cycle: u64) {}
    fn irq_pending(&self) -> bool {
        false
    }
    fn reset(&mut self) {}
}

pub fn create(id: u8) -> Result<Box<dyn Mapper>, String> {
    Ok(match id {
        0 => Box::new(mapper0::Mapper0),
        1 => Box::new(mapper1::Mapper1::default()),
        2 => Box::new(mapper2::Mapper2::default()),
        3 => Box::new(mapper3::Mapper3::default()),
        4 => Box::new(mapper4::Mapper4::default()),
        7 => Box::new(mapper7::Mapper7::default()),
        66 => Box::new(mapper66::Mapper66::default()),
        _ => {
            return Err(format!(
                "Unsupported mapper {id}; supported: 0, 1, 2, 3, 4, 7, 66"
            ));
        }
    })
}
