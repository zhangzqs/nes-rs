use core::panic;
use std::{cell::RefCell, rc::Rc};

use crate::{Bus, BusAdapter, Cartridge, Mirroring, Ram, Reader, Writer};

pub trait Ppu {
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

pub struct PpuBusAdapterForCpuBus(pub Rc<RefCell<dyn Ppu>>);

impl Reader for PpuBusAdapterForCpuBus {
    fn read(&self, addr: u16) -> u8 {
        match (addr - 0x2000) % 8 {
            2 => self.0.borrow().read_reg_status(),
            4 => self.0.borrow().read_reg_oam_data(),
            7 => self.0.borrow().read_reg_data(),
            _ => panic!("PPU read from unsupported address: {:#04X}", addr),
        }
    }
}

impl Writer for PpuBusAdapterForCpuBus {
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

impl BusAdapter for PpuBusAdapterForCpuBus {
    fn address_accept(&self, addr: u16) -> bool {
        return addr >= 0x2000 && addr < 0x4000;
    }
}

/// 游戏卡带中的图案表适配器，用于将游戏卡带的CHR-ROM区域映射到 PPU 总线上
/// 寻址范围是 [0x0000, 0x1FFF]
pub struct PatternTablesAdapterForPpuBus(pub Rc<RefCell<dyn Cartridge>>);

impl Reader for PatternTablesAdapterForPpuBus {
    fn read(&self, addr: u16) -> u8 {
        self.0.borrow().ppu_read(addr)
    }
}

impl Writer for PatternTablesAdapterForPpuBus {
    fn write(&mut self, addr: u16, data: u8) {
        self.0.borrow_mut().ppu_write(addr, data);
    }
}

impl BusAdapter for PatternTablesAdapterForPpuBus {
    fn address_accept(&self, addr: u16) -> bool {
        return addr < 0x2000;
    }
}

/// 名称表适配器，读取挂载在PPU总线上的VRAM中的名称表数据
/// 寻址范围是 [0x2000, 0x3EFF], 其中[0x3000, 0x3EFF]是镜像了[0x2000, 0x2FFF]的区域
/// VRAM 中的名称表区域总计有0x800=2KB字节
/// 每个名称表占用0x400字节，总共有4个名称表
/// 游戏画面分割成了32x30个图案块
/// 名称表用于确定画面中的每个图案块是什么，使用哪个8x8点阵
pub struct NameTablesAdapterForPpuBus {
    pub vram: Rc<RefCell<dyn Ram>>,
    pub mirroring: Mirroring,
}

impl NameTablesAdapterForPpuBus {
    fn mirror_address(&self, addr: u16) -> u16 {
        let base_addr = addr - 0x2000; // 转换为[0x0000, 0x1FFF]地址范围
        let base_addr = base_addr % 0x1000; // 将镜像地址进行映射到真正的数据区域
        let nametable_index = base_addr / 0x400; // 计算名称表索引 0,1,2,3
        let nametable_offset = base_addr % 0x400; // 计算名称表内的偏移地址
        match self.mirroring {
            Mirroring::Horizontal => {
                // NT0 = NT2
                // NT1 = NT3
                let mapping_index = if nametable_index % 2 == 0 { 0 } else { 1 };
                mapping_index * 0x400 + nametable_offset
            }
            Mirroring::Vertical => {
                // NT0 = NT1
                // NT2 = NT3
                let mapping_index = if nametable_index < 2 { 0 } else { 1 };
                mapping_index * 0x400 + nametable_offset
            }
            Mirroring::SingleScreen => {
                // 所有名称表都映射到 NT0
                nametable_offset
            }
            Mirroring::FourScreen => {
                // 四屏模式下，所有名称表都独立
                base_addr
            }
        }
    }
}

impl Reader for NameTablesAdapterForPpuBus {
    fn read(&self, addr: u16) -> u8 {
        let mirrored_addr = self.mirror_address(addr);
        self.vram.borrow().read(mirrored_addr)
    }
}

impl Writer for NameTablesAdapterForPpuBus {
    fn write(&mut self, addr: u16, data: u8) {
        let mirrored_addr = self.mirror_address(addr);
        self.vram.borrow_mut().write(mirrored_addr, data);
    }
}

impl BusAdapter for NameTablesAdapterForPpuBus {
    fn address_accept(&self, addr: u16) -> bool {
        return addr >= 0x2000 && addr < 0x3F00;
    }
}

/// 调色板适配器，读取挂载在PPU总线上的VRAM中的调色板数据
/// PPU 寻址范围是[0x3F00, 0x3FFF]
/// 调色板区域总计有0x20=32个字节
/// NES 有8个调色板，前4个是背景色，后4个是精灵色
/// 每个调色板有4个颜色，总计有4*8=32个颜色
/// 每个颜色是一个u8的索引，指向NES PPU的全局颜色表，可确定最终显示的颜色
pub struct PalettesTablesAdapterForPpuBus {
    pub vram: Rc<RefCell<dyn Ram>>,
}

impl Reader for PalettesTablesAdapterForPpuBus {
    fn read(&self, addr: u16) -> u8 {
        self.vram.borrow().read((addr - 0x3F00) % 0x20)
    }
}

impl Writer for PalettesTablesAdapterForPpuBus {
    fn write(&mut self, addr: u16, data: u8) {
        self.vram.borrow_mut().write((addr - 0x3F00) % 0x20, data);
    }
}

impl BusAdapter for PalettesTablesAdapterForPpuBus {
    fn address_accept(&self, addr: u16) -> bool {
        return addr >= 0x3F00 && addr < 0x4000;
    }
}

/// PPU 总线镜像适配器，用于在总线上构造出按0x4000镜像的PPU总线
/// 这使得PPU总线的地址范围可以存在如下映射：
/// [0x0000, 0x3FFF] => [0x0000,0x3FFF]
/// [0x4000, 0x7FFF] => [0x0000,0x3FFF]
/// [0x8000, 0xBFFF] => [0x0000,0x3FFF]
/// [0xC000, 0xFFFF] => [0x0000,0x3FFF]
pub struct MirrorBusAdapterForPpuBus(pub Rc<RefCell<dyn Bus>>);

impl Reader for MirrorBusAdapterForPpuBus {
    fn read(&self, addr: u16) -> u8 {
        self.0.borrow().read(addr % 0x4000)
    }
}

impl Writer for MirrorBusAdapterForPpuBus {
    fn write(&mut self, addr: u16, data: u8) {
        self.0.borrow_mut().write(addr % 0x4000, data);
    }
}

impl BusAdapter for MirrorBusAdapterForPpuBus {
    fn address_accept(&self, addr: u16) -> bool {
        addr >= 0x4000
    }
}
