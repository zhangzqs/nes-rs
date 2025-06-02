#[derive(Debug)]
pub enum Interrupt {
    Nmi,
    Reset,
    Irq,
}

pub trait CPU {
    fn total_cycles(&self) -> u32;
    fn remaining_cycles(&self) -> u32;
    fn increase_cycles(&mut self, cycles: u32);
    fn send_interrupt(&mut self, interrupt: Interrupt);
    fn clock(&mut self);
}
