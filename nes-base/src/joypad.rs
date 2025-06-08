use std::{cell::RefCell, rc::Rc};

use crate::{BusAdapter, Reader, Writer};

pub trait Joypad {
    /// CPU 写入选通信号
    fn write_reg_strobe(&mut self, value: u8);
    /// CPU 读取按键状态
    fn read_reg_key_state(&self) -> u8;
}

pub struct JoypadAdapterForCPUBus {
    pub joypad1: Option<Rc<RefCell<dyn Joypad>>>,
    pub joypad2: Option<Rc<RefCell<dyn Joypad>>>,
}

impl Reader for JoypadAdapterForCPUBus {
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

impl Writer for JoypadAdapterForCPUBus {
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4016 => {
                if let Some(joypad) = &self.joypad1 {
                    joypad.borrow_mut().write_reg_strobe(data);
                }
            }
            0x4017 => {
                if let Some(joypad) = &self.joypad2 {
                    joypad.borrow_mut().write_reg_strobe(data);
                }
            }
            _ => panic!("Joypad write to unsupported address: {:#04X}", addr),
        }
    }
}

impl BusAdapter for JoypadAdapterForCPUBus {
    fn address_accept(&self, addr: u16) -> bool {
        addr == 0x4016 || addr == 0x4017
    }
}
