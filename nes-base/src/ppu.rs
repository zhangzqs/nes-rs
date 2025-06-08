use std::{cell::RefCell, rc::Rc};

use crate::{Bus, BusAdapter, Cartridge, Mirroring, RAM, Reader, Writer};

pub trait PPU {
    fn write_reg_control(&mut self, value: u8);

    fn write_reg_mask(&mut self, value: u8);

    fn read_reg_status(&self) -> u8;

    fn write_reg_oam_addr(&mut self, value: u8);

    fn read_reg_oam_data(&self) -> u8;
    fn write_reg_oam_data(&mut self, value: u8);

    fn write_reg_scroll(&mut self, value: u8);

    fn write_reg_address(&mut self, value: u8);

    fn read_reg_data(&self) -> u8;
    fn write_reg_data(&mut self, value: u8);

    fn reset(&mut self);
    fn clock(&mut self);
    fn attach_bus(&mut self, bus: Rc<RefCell<dyn BusAdapter>>);
}

pub struct PPUBusAdapterForCPUBus(pub Rc<RefCell<dyn PPU>>);

impl Reader for PPUBusAdapterForCPUBus {
    fn read(&self, addr: u16) -> u8 {
        match (addr - 0x2000) % 8 {
            2 => self.0.borrow().read_reg_status(),
            4 => self.0.borrow().read_reg_oam_data(),
            7 => self.0.borrow().read_reg_data(),
            _ => panic!("PPU read from unsupported address: {:#04X}", addr),
        }
    }
}

impl Writer for PPUBusAdapterForCPUBus {
    fn write(&mut self, addr: u16, data: u8) {
        match (addr - 0x2000) % 8 {
            0 => self.0.borrow_mut().write_reg_control(data),
            1 => self.0.borrow_mut().write_reg_mask(data),
            3 => self.0.borrow_mut().write_reg_oam_addr(data),
            4 => self.0.borrow_mut().write_reg_oam_data(data),
            5 => self.0.borrow_mut().write_reg_scroll(data),
            6 => self.0.borrow_mut().write_reg_address(data),
            7 => self.0.borrow_mut().write_reg_data(data),
            _ => panic!("PPU write to unsupported address: {:#04X}", addr),
        }
    }
}

impl BusAdapter for PPUBusAdapterForCPUBus {
    fn address_accept(&self, addr: u16) -> bool {
        return addr >= 0x2000 && addr < 0x4000;
    }
}

/// 游戏卡带中的图案表适配器，用于 PPU 总线
pub struct PatternTablesBusAdapterForPPUBus(pub Rc<RefCell<dyn Cartridge>>);

impl Reader for PatternTablesBusAdapterForPPUBus {
    fn read(&self, addr: u16) -> u8 {
        self.0.borrow().ppu_read(addr)
    }
}

impl Writer for PatternTablesBusAdapterForPPUBus {
    fn write(&mut self, addr: u16, data: u8) {
        self.0.borrow_mut().ppu_write(addr, data);
    }
}

impl BusAdapter for PatternTablesBusAdapterForPPUBus {
    fn address_accept(&self, addr: u16) -> bool {
        return addr < 0x2000;
    }
}

/// 名称表适配器，读取挂载在PPU总线上的VRAM中的名称表数据
pub struct NameTablesForPPUBus {
    pub vram: Rc<RefCell<dyn RAM>>,
    pub mirroring: Mirroring,
}

impl NameTablesForPPUBus {
    fn mirror_address(&self, addr: u16) -> u16 {
        const MIRROR_LOOK_UP: [[u8; 4]; 4] = [
            [0, 0, 1, 1], //horizontal
            [0, 1, 0, 1], //vertical
            [0, 0, 0, 0], //singleScreen
            [1, 1, 1, 1], //fourScreen
        ];
        let mode = match self.mirroring {
            Mirroring::Horizontal => 0,
            Mirroring::Vertical => 1,
            Mirroring::SingleScreen => 2,
            Mirroring::FourScreen => 3,
        };
        let addr = (addr - 0x2000) % 0x1000;
        let bank = (addr / 0x400) as usize;
        let offset = (addr % 0x400) as usize;
        let mirrored_bank = MIRROR_LOOK_UP[mode][bank];
        0x2000 + mirrored_bank as u16 * 0x400 + offset as u16
    }
}
