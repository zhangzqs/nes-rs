use std::{cell::RefCell, rc::Rc};

use crate::{BusAdapter, Reader, Writer};

pub trait APU {
    // Pulse1 registers
    fn write_reg_pulse1_control(&mut self, value: u8);
    fn write_reg_pulse1_sweep(&mut self, value: u8);
    fn write_reg_pulse1_timer_low(&mut self, value: u8);
    fn write_reg_pulse1_timer_high(&mut self, value: u8);

    // Pulse2 registers
    fn write_reg_pulse2_control(&mut self, value: u8);
    fn write_reg_pulse2_sweep(&mut self, value: u8);
    fn write_reg_pulse2_timer_low(&mut self, value: u8);
    fn write_reg_pulse2_timer_high(&mut self, value: u8);

    // Triangle registers
    fn write_reg_triangle_control(&mut self, value: u8);
    fn write_reg_triangle_timer_low(&mut self, value: u8);
    fn write_reg_triangle_timer_high(&mut self, value: u8);

    // Noise registers
    fn write_reg_noise_control(&mut self, value: u8);
    fn write_reg_noise_period(&mut self, value: u8);
    fn write_reg_noise_length(&mut self, value: u8);

    // DMC registers
    fn write_reg_dmc_control(&mut self, value: u8);
    fn write_reg_dmc_value(&mut self, value: u8);
    fn write_reg_dmc_address(&mut self, value: u8);
    fn write_reg_dmc_length(&mut self, value: u8);

    // Other registers
    fn write_reg_control(&mut self, value: u8);
    fn write_reg_frame_counter(&mut self, value: u8);
    fn read_reg_status(&self) -> u8;

    fn clock(&mut self);
}

pub struct APUAdapterForCPUBus(pub Rc<RefCell<dyn APU>>);

impl Reader for APUAdapterForCPUBus {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x4015 => self.0.borrow().read_reg_status(),
            x => {
                panic!("APU read from unsupported address: {:#04X}", x);
            }
        }
    }
}

impl Writer for APUAdapterForCPUBus {
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000 => self.0.borrow_mut().write_reg_pulse1_control(data),
            0x4001 => self.0.borrow_mut().write_reg_pulse1_sweep(data),
            0x4002 => self.0.borrow_mut().write_reg_pulse1_timer_low(data),
            0x4003 => self.0.borrow_mut().write_reg_pulse1_timer_high(data),

            0x4004 => self.0.borrow_mut().write_reg_pulse2_control(data),
            0x4005 => self.0.borrow_mut().write_reg_pulse2_sweep(data),
            0x4006 => self.0.borrow_mut().write_reg_pulse2_timer_low(data),
            0x4007 => self.0.borrow_mut().write_reg_pulse2_timer_high(data),

            0x4008 => self.0.borrow_mut().write_reg_triangle_control(data),
            0x4009 => todo!("Unsupported APU write to 0x4009"),
            0x400A => self.0.borrow_mut().write_reg_triangle_timer_low(data),
            0x400B => self.0.borrow_mut().write_reg_triangle_timer_high(data),

            0x400C => self.0.borrow_mut().write_reg_noise_control(data),
            0x400D => todo!("Unsupported APU write to 0x400D"),
            0x400E => self.0.borrow_mut().write_reg_noise_period(data),
            0x400F => self.0.borrow_mut().write_reg_noise_length(data),

            0x4010 => self.0.borrow_mut().write_reg_dmc_control(data),
            0x4011 => self.0.borrow_mut().write_reg_dmc_value(data),
            0x4012 => self.0.borrow_mut().write_reg_dmc_address(data),
            0x4013 => self.0.borrow_mut().write_reg_dmc_length(data),

            0x4015 => self.0.borrow_mut().write_reg_control(data),
            0x4017 => self.0.borrow_mut().write_reg_frame_counter(data),

            addr => panic!("APU write to unsupported address: {:#04X}", addr),
        }
    }
}

impl BusAdapter for APUAdapterForCPUBus {
    fn address_accept(&self, addr: u16) -> bool {
        return addr >= 0x4000 && addr < 0x4014 || addr == 0x4015;
    }
}
