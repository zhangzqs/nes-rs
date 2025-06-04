use nes_base::{BusAdapter, CPU, CPUState, Interrupt};
use state::{Context, execute_instruction, execute_interrupt, get_data_address};
use std::{cell::RefCell, rc::Rc};

mod common;
mod opcode;
mod state;

pub struct CPUImpl {
    context: Context,
    interrupt: Option<Interrupt>,
    total_cycles: u32,
}

impl CPUImpl {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
            interrupt: None,
            total_cycles: 0,
        }
    }
}

impl CPU for CPUImpl {
    fn increase_cycles(&mut self, cycles: u32) {
        self.context.remaining_cycles += cycles;
    }

    fn send_interrupt(&mut self, interrupt: Interrupt) {
        if self.interrupt.is_none() {
            self.interrupt = Some(interrupt);
            return;
        }
        // 如果已经有中断在等待，则不处理新的中断
        // 这可以防止在处理中断时再次触发中断
        // 例如在 NMI 中断处理期间不允许 IRQ 中断
        log::warn!(
            "CPU already has an interrupt pending, ignoring new interrupt: {:?}",
            interrupt
        );
    }

    fn clock(&mut self) {
        // 执行周期
        if self.context.remaining_cycles > 0 {
            self.context.remaining_cycles -= 1;
            self.total_cycles += 1;
            return;
        }

        // 中断周期
        if let Some(interrupt) = self.interrupt.take() {
            execute_interrupt(&mut self.context, interrupt);
            self.interrupt = None;
            return;
        }

        // 取指周期
        let op = opcode::get_op(self.context.get_opcode());
        self.increase_cycles(op.cycles as u32);

        // 寻址
        let result = get_data_address(&self.context);
        self.context.data_address = result.address;
        self.context.increase_pc(result.pc_increment);
        if result.page_crossed && op.increase_cycle_when_cross_page {
            self.increase_cycles(1);
        }

        // 执行指令
        execute_instruction(&mut self.context, op.instruction);
    }

    fn dump_state(&self) -> CPUState {
        CPUState {
            total_cycles: self.total_cycles,
            remaining_cycles: self.context.remaining_cycles,
            reg_a: self.context.reg_a,
            reg_x: self.context.reg_x,
            reg_y: self.context.reg_y,
            reg_sp: self.context.reg_sp,
            reg_pc: self.context.reg_pc,
            reg_status: self.context.reg_status,
        }
    }

    fn attach_bus(&mut self, bus: Rc<RefCell<dyn BusAdapter>>) {
        self.context.bus = Some(bus);
    }
}
