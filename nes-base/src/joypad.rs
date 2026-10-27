use std::{cell::RefCell, rc::Rc};

use crate::{BusAdapter, Reader, Writer};

pub trait Joypad {
    /// CPU 写入选通信号
    fn write_reg_strobe(&mut self, value: u8);
    /// CPU 读取按键状态
    fn read_reg_key_state(&self) -> u8;
}

pub struct JoypadAdapterForCpuBus {
    pub joypad1: Option<Rc<RefCell<dyn Joypad>>>,
    pub joypad2: Option<Rc<RefCell<dyn Joypad>>>,
    pub apu: Rc<RefCell<dyn crate::Apu>>,
}

impl Reader for JoypadAdapterForCpuBus {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x4016 => {
                if let Some(joypad) = &self.joypad1 {
                    joypad.borrow().read_reg_key_state()
                } else {
                    0
                }
            }
            0x4017 => {
                if let Some(joypad) = &self.joypad2 {
                    joypad.borrow().read_reg_key_state()
                } else {
                    0
                }
            }
            _ => panic!("Joypad read from unsupported address: {:#04X}", addr),
        }
    }
}

impl Writer for JoypadAdapterForCpuBus {
    fn write(&mut self, addr: u16, data: u8) {
        if addr == 0x4016 {
            for joypad in [&self.joypad1, &self.joypad2].into_iter().flatten() {
                joypad.borrow_mut().write_reg_strobe(data);
            }
        } else if addr == 0x4017 {
            self.apu.borrow_mut().write_reg_frame_counter(data);
        }
    }
}

impl BusAdapter for JoypadAdapterForCpuBus {
    fn address_accept(&self, addr: u16) -> bool {
        addr == 0x4016 || addr == 0x4017
    }
}
