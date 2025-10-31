use std::cell::{Cell, RefCell};
use std::rc::Rc;

use nes_base::{BusAdapter, PPU_FRAME_PIXELS, Ppu};

use crate::oam::Oam;
use crate::register::{
    PpuAddressRegister, PpuControlRegister, PpuMaskRegister, PpuScrollRegister, PpuStatusRegister,
};

pub use self::framebuffer::{FRAME_HEIGHT, FRAME_PIXELS, FRAME_WIDTH, FrameBuffer};
pub use palettes::NES_SYS_PALETTES;

mod framebuffer;
mod oam;
pub mod palettes;
pub mod register;
mod renderer;

pub struct PpuImpl {
    ppu_bus: Option<Rc<RefCell<dyn BusAdapter>>>,
    scanline: u16,
    cycle: u16,
    frame_counter: u64,
    nmi_interrupt: bool,
    oam: Oam,
    reg_ppu_controller: PpuControlRegister,
    reg_ppu_mask: PpuMaskRegister,
    reg_ppu_status: RefCell<PpuStatusRegister>,
    reg_scroll: RefCell<PpuScrollRegister>,
    reg_address: RefCell<PpuAddressRegister>,
    reg_oam_address: u8,
    vram_read_buffer: Cell<u8>,
    frame_buffer: RefCell<FrameBuffer>,
    frame_ready: Cell<bool>,
}

impl PpuImpl {
    pub fn new() -> Self {
        Self {
            ppu_bus: None,
            scanline: 261, // pre-render scanline
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
            vram_read_buffer: Cell::new(0),
            frame_buffer: RefCell::new(FrameBuffer::new()),
            frame_ready: Cell::new(false),
        }
    }

    fn increment_address(&self) {
        let increment = self.reg_ppu_controller.vram_increment_value();
        self.reg_address.borrow_mut().increment(increment);
    }

    fn render_current_frame(&mut self) {
        let Some(bus) = &self.ppu_bus else {
            return;
        };
        let mut framebuffer = self.frame_buffer.borrow_mut();
        framebuffer.clear();
        let sprite0_hit = renderer::render_frame(self, &*bus.borrow(), &mut framebuffer);
        self.reg_ppu_status.borrow_mut().sprite_0_hit = sprite0_hit;
        self.frame_ready.set(true);
    }

    pub(crate) fn control(&self) -> PpuControlRegister {
        self.reg_ppu_controller
    }

    pub(crate) fn mask(&self) -> PpuMaskRegister {
        self.reg_ppu_mask
    }

    pub(crate) fn scroll(&self) -> PpuScrollRegister {
        *self.reg_scroll.borrow()
    }

    pub(crate) fn oam(&self) -> &Oam {
        &self.oam
    }
}

impl Ppu for PpuImpl {
    fn write_reg_control(&mut self, value: u8) {
        let before_nmi_status = self.reg_ppu_controller.nmi_enable;
        self.reg_ppu_controller = PpuControlRegister::from(value);
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
        let value: u8 = self.reg_ppu_status.borrow().clone().into();
        self.reg_ppu_status.borrow_mut().vblank = false;
        self.reg_address.borrow_mut().reset_latch();
        self.reg_scroll.borrow_mut().reset_latch();
        value
    }

    fn write_reg_oam_addr(&mut self, value: u8) {
        self.reg_oam_address = value;
    }

    fn read_reg_oam_data(&self) -> u8 {
        self.oam.read(self.reg_oam_address)
    }

    fn write_reg_oam_data(&mut self, value: u8) {
        self.oam.write(self.reg_oam_address, value);
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
        let bus = self.ppu_bus.as_ref().expect("PPU bus not attached");
        let bus_ref = bus.borrow();

        let previous = self.vram_read_buffer.get();
        let value = bus_ref.read(addr);
        let result = if addr >= 0x3F00 {
            let mirrored_addr = (addr - 0x1000) & 0x3FFF;
            let buffered = bus_ref.read(mirrored_addr);
            self.vram_read_buffer.set(buffered);
            value
        } else {
            self.vram_read_buffer.set(value);
            previous
        };

        self.increment_address();
        result
    }

    fn write_reg_data(&mut self, value: u8) {
        let addr = self.reg_address.borrow().get();
        let bus = self.ppu_bus.as_ref().expect("PPU bus not attached");
        bus.borrow_mut().write(addr, value);
        self.increment_address();
    }

    fn reset(&mut self) {
        self.scanline = 261;
        self.cycle = 0;
        self.frame_counter = 0;
        self.nmi_interrupt = false;
        self.oam = Oam::new();
        self.reg_ppu_controller = PpuControlRegister::default();
        self.reg_ppu_mask = PpuMaskRegister::default();
        *self.reg_ppu_status.borrow_mut() = PpuStatusRegister::default();
        self.reg_scroll.borrow_mut().reset_latch();
        self.reg_address.borrow_mut().reset_latch();
        self.reg_oam_address = 0;
        self.vram_read_buffer.set(0);
        self.frame_buffer.borrow_mut().clear();
        self.frame_ready.set(false);
    }

    fn clock(&mut self) {
        self.cycle += 1;

        if self.scanline == 241 && self.cycle == 1 {
            self.reg_ppu_status.borrow_mut().vblank = true;
            if self.reg_ppu_controller.nmi_enable {
                self.nmi_interrupt = true;
            }
            self.render_current_frame();
        }

        if self.scanline == 261 && self.cycle == 1 {
            self.reg_ppu_status.borrow_mut().vblank = false;
            self.reg_ppu_status.borrow_mut().sprite_0_hit = false;
            self.reg_ppu_status.borrow_mut().sprite_overflow = false;
            self.nmi_interrupt = false;
            self.frame_ready.set(false);
        }

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline = (self.scanline + 1) % 262;
            if self.scanline == 0 {
                self.frame_counter = self.frame_counter.wrapping_add(1);
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

    fn frame_ready(&self) -> bool {
        self.frame_ready.get()
    }

    fn copy_frame(&self, target: &mut [u8]) -> bool {
        if !self.frame_ready.get() || target.len() < PPU_FRAME_PIXELS {
            return false;
        }
        let framebuffer = self.frame_buffer.borrow();
        target[..PPU_FRAME_PIXELS].copy_from_slice(framebuffer.pixels());
        true
    }

    fn acknowledge_frame(&mut self) {
        self.frame_ready.set(false);
    }
}
