use std::{cell::RefCell, rc::Rc};

use nes_base::{Apu, Ppu};
use nes_board::BoardImpl;
use nes_bus::BusImpl;
use nes_cpu::CpuImpl;

mod neslog;

#[cfg(test)]
mod cpu_tests;

#[cfg(test)]
mod tile_tests;

struct MockPPU;

impl Ppu for MockPPU {
    fn write_reg_control(&mut self, _value: u8) {}

    fn write_reg_mask(&mut self, _value: u8) {}

    fn read_reg_status(&self) -> u8 {
        0
    }

    fn write_reg_oam_addr(&mut self, _value: u8) {}

    fn read_reg_oam_data(&self) -> u8 {
        0
    }

    fn write_reg_oam_data(&mut self, _value: u8) {}

    fn write_reg_scroll(&mut self, _value: u8) {}

    fn write_reg_address(&mut self, _value: u8) {}

    fn read_reg_data(&self) -> u8 {
        0
    }

    fn write_reg_data(&mut self, _value: u8) {}

    fn reset(&mut self) {}

    fn clock(&mut self) {}

    fn attach_bus(&mut self, bus: std::rc::Rc<std::cell::RefCell<dyn nes_base::BusAdapter>>) {}
}

struct MockAPU;

impl Apu for MockAPU {
    fn write_reg_pulse1_control(&mut self, _value: u8) {}

    fn write_reg_pulse1_sweep(&mut self, _value: u8) {}

    fn write_reg_pulse1_timer_low(&mut self, _value: u8) {}

    fn write_reg_pulse1_timer_high(&mut self, _value: u8) {}

    fn write_reg_pulse2_control(&mut self, _value: u8) {}

    fn write_reg_pulse2_sweep(&mut self, _value: u8) {}

    fn write_reg_pulse2_timer_low(&mut self, _value: u8) {}

    fn write_reg_pulse2_timer_high(&mut self, _value: u8) {}

    fn write_reg_triangle_control(&mut self, _value: u8) {}

    fn write_reg_triangle_timer_low(&mut self, _value: u8) {}

    fn write_reg_triangle_timer_high(&mut self, _value: u8) {}

    fn write_reg_noise_control(&mut self, _value: u8) {}

    fn write_reg_noise_period(&mut self, _value: u8) {}

    fn write_reg_noise_length(&mut self, _value: u8) {}

    fn write_reg_dmc_control(&mut self, _value: u8) {}

    fn write_reg_dmc_value(&mut self, _value: u8) {}

    fn write_reg_dmc_address(&mut self, _value: u8) {}

    fn write_reg_dmc_length(&mut self, _value: u8) {}

    fn write_reg_control(&mut self, _value: u8) {}

    fn write_reg_frame_counter(&mut self, _value: u8) {}

    fn read_reg_status(&self) -> u8 {
        0
    }

    fn clock(&mut self) {}
}

fn new_board() -> BoardImpl {
    let nes = nes_cartridge::NESFile::from_file("testfiles/nestest.nes");
    let cartridge = nes_cartridge::CartridgeImpl::new(nes);
    let board = BoardImpl {
        joypad1: None,
        joypad2: None,
        cpu_bus: Rc::new(RefCell::new(BusImpl::new())),
        ppu_bus: Rc::new(RefCell::new(BusImpl::new())),
        cpu: Rc::new(RefCell::new(CpuImpl::new())),
        ppu: Rc::new(RefCell::new(MockPPU)),
        apu: Rc::new(RefCell::new(MockAPU)),
        ram: Rc::new(RefCell::new(nes_ram::RamImpl::new(0x800))),
        cartridge: Rc::new(RefCell::new(cartridge)),
    }
    .init();
    board
}
