use std::{cell::RefCell, rc::Rc};

use crate::BusAdapter;

#[derive(Debug)]
pub enum Interrupt {
    Nmi,
    Reset,
    Irq,
}

#[derive(Debug)]
pub struct CpuState {
    pub total_cycles: u32,
    pub remaining_cycles: u32,
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub reg_sp: u8,
    pub reg_pc: u16,
    pub reg_status: CpuStatusFlags,
}

#[derive(Debug, Clone, Copy)]
pub struct CpuStatusFlags {
    pub carry: bool,
    pub zero: bool,
    pub interrupt_disable: bool,
    pub decimal_mode: bool,
    pub break_command: bool,
    pub unused: bool, // 这个位在 6502 中未使用，但在某些指令中会被设置
    pub overflow: bool,
    pub negative: bool,
}

impl From<u8> for CpuStatusFlags {
    fn from(value: u8) -> Self {
        Self {
            carry: value & 0b0000_0001 != 0,
            zero: value & 0b0000_0010 != 0,
            interrupt_disable: value & 0b0000_0100 != 0,
            decimal_mode: value & 0b0000_1000 != 0,
            break_command: value & 0b0001_0000 != 0,
            unused: value & 0b0010_0000 != 0, // 在 6502 中未使用，但在某些指令中会被设置
            overflow: value & 0b0100_0000 != 0,
            negative: value & 0b1000_0000 != 0,
        }
    }
}

impl From<CpuStatusFlags> for u8 {
    fn from(val: CpuStatusFlags) -> Self {
        (val.carry as u8)
            | ((val.zero as u8) << 1)
            | ((val.interrupt_disable as u8) << 2)
            | ((val.decimal_mode as u8) << 3)
            | ((val.break_command as u8) << 4)
            | ((val.unused as u8) << 5)
            | ((val.overflow as u8) << 6)
            | ((val.negative as u8) << 7)
    }
}

pub trait Cpu {
    fn set_reg_pc(&mut self, pc: u16);
    fn reset(&mut self);
    fn attach_bus(&mut self, bus: Rc<RefCell<dyn BusAdapter>>);
    fn dump_state(&self) -> CpuState;
    fn increase_cycles(&mut self, cycles: u32);
    fn trigger_interrupt(&mut self, interrupt: Interrupt);
    fn clock(&mut self);
}
