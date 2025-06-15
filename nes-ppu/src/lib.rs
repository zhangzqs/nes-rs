use std::{cell::RefCell, rc::Rc};

use nes_base::{BusAdapter, Ppu};

use crate::{
    oam::Oam,
    register::{
        PpuAddressRegister, PpuControlRegister, PpuMaskRegister, PpuScrollRegister,
        PpuStatusRegister,
    },
};

mod framebuffer;
mod oam;
mod palettes;
mod register;
mod renderer;

pub struct PpuImpl {
    /// ```text
    /// PPU Addrress Mapping:
    /// [0x0000, 0x2000) CHR-ROM Pattern Table
    /// [0x2000, 0x3000) NameTable RAM 0x400 bytes
    ///    [0x2000, 0x2400) Name Table 0
    ///    [0x2400, 0x2800) Name Table 1
    ///    [0x2800, 0x2C00) Name Table 2
    ///    [0x2C00, 0x3000) Name Table 3
    /// [0x3000, 0x4000) NameTable RAM mirror
    /// [0x3F00, 0x4000) PaletteTable RAM 0x20 bytes
    ///   [0x3F00, 0x3F20) Palette
    ///   [0x3F20, 0x4000) Palette mirror
    /// [0x4000, 0x10000) Mirror [0x0000,0x4000)
    /// ``````
    ppu_bus: Option<Rc<RefCell<dyn BusAdapter>>>,
    scanline: u16,
    cycle: u16,
    frame_counter: u32,
    nmi_interrupt: bool,
    oam: Oam,

    // PPU 的8个寄存器
    reg_ppu_controller: PpuControlRegister,
    reg_ppu_mask: PpuMaskRegister,
    reg_ppu_status: RefCell<PpuStatusRegister>,
    reg_scroll: RefCell<PpuScrollRegister>,
    reg_address: RefCell<PpuAddressRegister>,
    reg_oam_address: u8,
    reg_oam_data: u8,
}

impl PpuImpl {
    fn new() -> Self {
        Self {
            ppu_bus: None,
            scanline: 0,
            cycle: 0,
            frame_counter: 0,
            nmi_interrupt: false,
            oam: Oam::new(),
            reg_ppu_controller: PpuControlRegister::default(),
            reg_ppu_mask: PpuMaskRegister::default(),
            reg_ppu_status: RefCell::new(PpuStatusRegister::default()),
            reg_scroll: RefCell::new(PpuScrollRegister::default()),
            reg_address: RefCell::new(PpuAddressRegister::default()),
            reg_oam_address: 0,
            reg_oam_data: 0,
        }
    }

    fn increment_address(&self) {
        let increment = self.reg_ppu_controller.vram_increment_value();
        self.reg_address.borrow_mut().increment(increment);
    }
}

impl Ppu for PpuImpl {
    fn write_reg_control(&mut self, value: u8) {
        let before_nmi_status = self.reg_ppu_controller.nmi_enable;
        // 更新控制寄存器
        self.reg_ppu_controller = PpuControlRegister::from(value);

        // 如果在vblank期间 NMI 使能状态从禁用变为使能，触发 NMI 中断
        if !before_nmi_status
            && self.reg_ppu_controller.nmi_enable
            && self.reg_ppu_status.borrow().vblank
        {
            self.nmi_interrupt = true;
        }
    }

    fn write_reg_mask(&mut self, value: u8) {
        self.reg_ppu_mask = PpuMaskRegister::from(value);
    }

    fn read_reg_status(&self) -> u8 {
        let value = self.reg_ppu_status.borrow().clone().into();
        self.reg_ppu_status.borrow_mut().vblank = false; // 读取后清除 VBlank 标志
        self.reg_address.borrow_mut().reset_latch();
        self.reg_scroll.borrow_mut().reset_latch();
        value
    }

    fn write_reg_oam_addr(&mut self, value: u8) {
        self.reg_oam_address = value;
    }

    fn read_reg_oam_data(&self) -> u8 {
        self.reg_oam_data
    }

    fn write_reg_oam_data(&mut self, value: u8) {
        self.reg_oam_data = value;
        self.reg_oam_address = self.reg_oam_address.wrapping_add(1);
    }

    fn write_reg_scroll(&mut self, value: u8) {
        self.reg_scroll.borrow_mut().write(value);
    }

    fn write_reg_address(&mut self, value: u8) {
        self.reg_address.borrow_mut().update(value);
    }

    fn read_reg_data(&self) -> u8 {
        let addr = self.reg_address.borrow().get();
        let value = self
            .ppu_bus
            .as_ref()
            .expect("PPU bus not attached")
            .borrow()
            .read(addr);

        // 更新地址
        self.increment_address();

        value
    }

    fn write_reg_data(&mut self, value: u8) {
        let addr = self.reg_address.borrow().get();
        self.ppu_bus
            .as_ref()
            .expect("PPU bus not attached")
            .borrow_mut()
            .write(addr, value);

        // 更新地址
        self.increment_address();
    }

    fn reset(&mut self) {
        self.cycle = 340;
        self.scanline = 240;
        self.frame_counter = 0;
        self.reg_ppu_controller = PpuControlRegister::default();
        self.reg_ppu_mask = PpuMaskRegister::default();
        *self.reg_ppu_status.borrow_mut() = PpuStatusRegister::default();
    }

    fn clock(&mut self) {
        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline == 241 {
                self.reg_ppu_status.borrow_mut().vblank = true;
                self.reg_ppu_status.borrow_mut().sprite_0_hit = false;
                if self.reg_ppu_controller.nmi_enable {
                    self.nmi_interrupt = true;
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.frame_counter += 1;
                self.nmi_interrupt = false;
                self.reg_ppu_status.borrow_mut().vblank = false;
                self.reg_ppu_status.borrow_mut().sprite_0_hit = false;
            }
        }
    }

    fn attach_bus(&mut self, bus: Rc<RefCell<dyn BusAdapter>>) {
        self.ppu_bus = Some(bus);
    }

    fn check_nmi_interrupt(&self) -> bool {
        self.nmi_interrupt
    }

    fn clear_nmi_interrupt(&mut self) {
        self.nmi_interrupt = false;
    }
}
