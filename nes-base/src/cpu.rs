use std::{cell::RefCell, rc::Rc};

use crate::BusAdapter;

#[derive(Debug)]
pub enum Interrupt {
    Nmi,
    Reset,
    Irq,
}

pub struct CPUState {
    pub total_cycles: u32,
    pub remaining_cycles: u32,
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub reg_sp: u8,
    pub reg_pc: u16,
    pub reg_status: u8,
}

pub trait CPU {
    fn set_reg_pc(&mut self, pc: u16);
    fn reset(&mut self);
    fn attach_bus(&mut self, bus: Rc<RefCell<dyn BusAdapter>>);
    fn dump_state(&self) -> CPUState;
    fn increase_cycles(&mut self, cycles: u32);
    fn send_interrupt(&mut self, interrupt: Interrupt);
    fn clock(&mut self);
}
