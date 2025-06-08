use log::debug;
use nes_base::{BusAdapter, Cpu, CpuState, Interrupt};
use state::{Context, execute_instruction, execute_interrupt, get_data_address};
use std::{cell::RefCell, rc::Rc};

mod common;
mod opcode;
mod state;

pub struct CpuImpl {
    context: Context,
    interrupt: Option<Interrupt>,
    total_cycles: u32,
}

impl Default for CpuImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuImpl {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
            interrupt: None,
            total_cycles: 0,
        }
    }
}

impl Cpu for CpuImpl {
    fn set_reg_pc(&mut self, pc: u16) {
        self.context.reg_pc = pc;
        self.context.remaining_cycles = 0; // 重置剩余周期
    }

    fn reset(&mut self) {
        execute_interrupt(&mut self.context, Interrupt::Reset);
        self.total_cycles = 7;
    }

    fn increase_cycles(&mut self, cycles: u32) {
        self.context.remaining_cycles += cycles;
    }

    fn trigger_interrupt(&mut self, interrupt: Interrupt) {
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

        // 取指 && 译码
        let reg_pc = self.context.reg_pc; // 取指
        let opcode = self.context.read_bus_8bit(reg_pc); // 从总线读取指令
        let op = opcode::get_op(opcode); // 指令译码
        self.context.op = Some(op);

        // 译码获取操作数地址，并更新pc
        let result = get_data_address(&self.context);
        self.context.data_address = result.address;
        self.context.reg_pc = self.context.reg_pc.wrapping_add(result.pc_increment);

        if result.page_crossed && op.increase_cycle_when_cross_page {
            debug!(
                "Page crossed during addressing, increasing cycles for instruction: {:?}",
                op.instruction
            );
            self.increase_cycles(1);
        }

        // 执行周期
        debug!(
            "Executing instruction: {:?}, PC: {:04X}, A: {:02X}, X: {:02X}, Y: {:02X}, SP: {:02X}, Status: {:02X}",
            op.instruction,
            self.context.reg_pc,
            self.context.reg_a,
            self.context.reg_x,
            self.context.reg_y,
            self.context.reg_sp,
            self.context.reg_status
        );
        self.increase_cycles(op.cycles as u32);
        execute_instruction(&mut self.context, op.instruction);
        debug!(
            "Instruction executed: {:?}, PC: {:04X}, A: {:02X}, X: {:02X}, Y: {:02X}, SP: {:02X}, Status: {:02X}",
            op.instruction,
            self.context.reg_pc,
            self.context.reg_a,
            self.context.reg_x,
            self.context.reg_y,
            self.context.reg_sp,
            self.context.reg_status
        );
    }

    fn dump_state(&self) -> CpuState {
        CpuState {
            total_cycles: self.total_cycles,
            remaining_cycles: self.context.remaining_cycles,
            reg_a: self.context.reg_a,
            reg_x: self.context.reg_x,
            reg_y: self.context.reg_y,
            reg_sp: self.context.reg_sp,
            reg_pc: self.context.reg_pc,
            reg_status: self.context.reg_status.into(),
        }
    }

    fn attach_bus(&mut self, bus: Rc<RefCell<dyn BusAdapter>>) {
        self.context.bus = Some(bus);
    }
}
