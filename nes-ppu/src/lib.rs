use nes_base::{BusAdapter, Ppu};
use oam::Oam;
use register::{PpuAddressRegister, PpuControlRegister, PpuMaskRegister, PpuStatusRegister};
use renderer::RenderState;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub mod framebuffer;
mod oam;
mod palettes;
mod register;
mod renderer;

pub use framebuffer::{FrameBuffer, HEIGHT, WIDTH};

pub struct PpuImpl {
    ppu_bus: Option<Rc<RefCell<dyn BusAdapter>>>,
    scanline: u16,
    cycle: u16,
    frame_counter: u64,
    total_cycles: u64,
    bus_address: Cell<u16>,
    odd_frame: bool,
    skip_frame_dot: bool,
    nmi_interrupt: Cell<bool>,
    nmi_delay: Cell<u8>,
    suppress_vblank: Cell<bool>,
    oam: Oam,
    control: PpuControlRegister,
    mask: PpuMaskRegister,
    status: Cell<PpuStatusRegister>,
    address: PpuAddressRegister,
    oam_address: u8,
    open_bus: Cell<u8>,
    read_buffer: Cell<u8>,
    framebuffer: FrameBuffer,
    renderer: RenderState,
}

impl Default for PpuImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl PpuImpl {
    pub fn new() -> Self {
        Self {
            ppu_bus: None,
            scanline: 261,
            cycle: 0,
            frame_counter: 0,
            odd_frame: false,
            skip_frame_dot: false,
            total_cycles: 0,
            bus_address: Cell::new(0),
            nmi_interrupt: Cell::new(false),
            nmi_delay: Cell::new(0),
            suppress_vblank: Cell::new(false),
            oam: Oam::new(),
            control: Default::default(),
            mask: Default::default(),
            status: Cell::new(PpuStatusRegister::default()),
            address: Default::default(),
            oam_address: 0,
            open_bus: Cell::new(0),
            read_buffer: Cell::new(0),
            framebuffer: FrameBuffer::new(),
            renderer: RenderState::default(),
        }
    }
    pub fn framebuffer(&self) -> &FrameBuffer {
        &self.framebuffer
    }
    pub fn frame_counter(&self) -> u64 {
        self.frame_counter
    }
    pub fn position(&self) -> (u16, u16) {
        (self.scanline, self.cycle)
    }

    fn observe_address(&self, address: u16) {
        self.bus_address.set(address & 0x3fff);
        self.ppu_bus
            .as_ref()
            .expect("PPU bus not attached")
            .borrow()
            .observe_ppu_address(address & 0x3fff, self.total_cycles);
    }
    fn read(&self, address: u16) -> u8 {
        self.ppu_bus
            .as_ref()
            .expect("PPU bus not attached")
            .borrow()
            .read(address & 0x3fff)
    }
    fn rendering(&self) -> bool {
        self.mask.show_background || self.mask.show_sprites
    }
    fn increment_address(&self) {
        if self.rendering() && (self.scanline < 240 || self.scanline == 261) {
            self.address.increment_x();
            self.address.increment_y();
        } else {
            self.address
                .v
                .set((self.address.v.get() + self.control.vram_increment_value() as u16) & 0x7fff);
        }
    }
}

impl Ppu for PpuImpl {
    fn read_open_bus(&self) -> u8 {
        self.open_bus.get()
    }
    fn write_open_bus(&mut self, value: u8) {
        self.open_bus.set(value);
    }
    fn write_reg_control(&mut self, value: u8) {
        self.open_bus.set(value);
        let old_nmi = self.control.nmi_enable;
        self.control = value.into();
        self.address.t = (self.address.t & !0x0c00) | ((value as u16 & 3) << 10);
        if !old_nmi && self.control.nmi_enable && self.status.get().vblank {
            self.nmi_delay.set(2);
        }
        if !self.control.nmi_enable {
            self.nmi_interrupt.set(false);
            self.nmi_delay.set(0);
        }
    }
    fn write_reg_mask(&mut self, value: u8) {
        self.open_bus.set(value);
        self.mask = value.into();
    }
    fn read_reg_status(&self) -> u8 {
        let status: u8 = self.status.get().into();
        let value = status | (self.open_bus.get() & 0x1f);
        let mut state = self.status.get();
        state.vblank = false;
        self.status.set(state);
        self.address.second_write.set(false);
        self.nmi_interrupt.set(false);
        self.nmi_delay.set(0);
        if self.scanline == 241 && self.cycle == 1 {
            self.suppress_vblank.set(true);
        }
        self.open_bus.set(value);
        value
    }
    fn write_reg_oam_addr(&mut self, value: u8) {
        self.open_bus.set(value);
        self.oam_address = value;
    }
    fn read_reg_oam_data(&self) -> u8 {
        let value = self.oam.data[self.oam_address as usize];
        self.open_bus.set(value);
        value
    }
    fn write_reg_oam_data(&mut self, value: u8) {
        self.open_bus.set(value);
        self.oam.data[self.oam_address as usize] = if self.oam_address & 3 == 2 {
            value & 0xe3
        } else {
            value
        };
        self.oam_address = self.oam_address.wrapping_add(1);
    }
    fn write_reg_scroll(&mut self, value: u8) {
        self.open_bus.set(value);
        self.address.write_scroll(value);
    }
    fn write_reg_address(&mut self, value: u8) {
        self.open_bus.set(value);
        self.address.write_address(value);
        if !self.address.second_write.get() && !self.rendering() {
            self.observe_address(self.address.v.get());
        }
    }
    fn read_reg_data(&self) -> u8 {
        let address = self.address.v.get() & 0x3fff;
        self.observe_address(address);
        let data = self.read(address);
        let value = if address >= 0x3f00 {
            self.read_buffer.set(self.read(address - 0x1000));
            let mask = if self.mask.grayscale { 0x30 } else { 0x3f };
            (data & mask) | (self.open_bus.get() & 0xc0)
        } else {
            self.read_buffer.replace(data)
        };
        self.open_bus.set(value);
        self.increment_address();
        self.observe_address(self.address.v.get());
        value
    }
    fn write_reg_data(&mut self, value: u8) {
        self.open_bus.set(value);
        self.observe_address(self.address.v.get());
        self.ppu_bus
            .as_ref()
            .expect("PPU bus not attached")
            .borrow_mut()
            .write(self.address.v.get() & 0x3fff, value);
        self.increment_address();
        self.observe_address(self.address.v.get());
    }
    fn reset(&mut self) {
        let bus = self.ppu_bus.take();
        *self = Self::new();
        self.ppu_bus = bus;
    }
    fn clock(&mut self) {
        // NMI 边沿经过两个 PPU 点时钟才传到 CPU；在此期间访问 $2002/$2000
        // 可以取消该边沿。已经锁存的边沿仍保持待响应状态。
        let delay = self.nmi_delay.get();
        if delay != 0 {
            self.nmi_delay.set(delay - 1);
            if delay == 1 {
                self.nmi_interrupt.set(true);
            }
        }
        if self.scanline == 261 && self.cycle == 1 {
            self.status.set(PpuStatusRegister::default());
            self.nmi_delay.set(0);
            self.suppress_vblank.set(false);
        }
        if self.scanline == 241 && self.cycle == 1 {
            if !self.suppress_vblank.get() {
                let mut status = self.status.get();
                status.vblank = true;
                self.status.set(status);
                if self.control.nmi_enable {
                    self.nmi_delay.set(2);
                }
            }
            self.frame_counter = self.frame_counter.wrapping_add(1);
        }
        self.render_clock();
        self.observe_address(self.bus_address.get());
        self.total_cycles = self.total_cycles.wrapping_add(1);
        // 在判断奇数帧是否提前结束的前一个点时钟，锁存渲染状态。
        if self.scanline == 261 && self.cycle == 338 {
            self.skip_frame_dot = self.rendering();
        }
        if self.scanline == 261 && self.cycle == 339 && self.odd_frame && self.skip_frame_dot {
            self.cycle = 0;
            self.scanline = 0;
            self.odd_frame = false;
        } else if self.cycle == 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline == 262 {
                self.scanline = 0;
                self.odd_frame = !self.odd_frame;
            }
        } else {
            self.cycle += 1;
        }
    }
    fn attach_bus(&mut self, bus: Rc<RefCell<dyn BusAdapter>>) {
        self.ppu_bus = Some(bus);
    }
    fn check_nmi_interrupt(&self) -> bool {
        self.nmi_interrupt.get()
    }
    fn clear_nmi_interrupt(&mut self) {
        self.nmi_interrupt.set(false);
        self.nmi_delay.set(0);
    }
}
