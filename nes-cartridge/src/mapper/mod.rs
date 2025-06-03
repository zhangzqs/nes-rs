use std::{cell::RefCell, rc::Rc};

use nes_base::{Cartridge, Memory};

use crate::mapper::{mapper0::Mapper0, mapper2::Mapper2};

mod mapper0;
mod mapper2;

pub fn get_mapper_by_id(
    mapper_id: u8,
    prg_banks: u8,
    chr_rom: Rc<RefCell<Vec<u8>>>,
    prg_rom: Rc<RefCell<Vec<u8>>>,
    sram: Option<Rc<RefCell<dyn Memory>>>,
) -> Box<dyn Cartridge> {
    match mapper_id {
        0 => Box::new(Mapper0::new(prg_banks, chr_rom, prg_rom, sram)),
        2 => Box::new(Mapper2::new(prg_banks, chr_rom, prg_rom, sram)),
        _ => panic!("Unsupported mapper ID: {}", mapper_id),
    }
}
