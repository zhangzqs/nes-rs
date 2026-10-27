use addressing::get_data_address;
use log::debug;
use nes_base::{BusAdapter, Cpu, CpuState, Interrupt};
use state::{Context, execute_instruction, execute_interrupt};
use std::{cell::RefCell, rc::Rc};

mod addressing;
mod common;
mod opcode;
mod state;

pub struct CpuImpl {
    context: Context,
    interrupt: Option<Interrupt>,
    total_cycles: u32,
    instruction_pending: bool,
    defer_interrupt: bool,
    irq_line: bool,
    irq_sampled: bool,
    halted: Option<(u16, u8)>,
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
            instruction_pending: false,
            defer_interrupt: false,
            irq_line: false,
            irq_sampled: false,
            halted: None,
        }
    }
}

impl Cpu for CpuImpl {
    fn halt_reason(&self) -> Option<(u16, u8)> {
        self.halted
    }
    fn set_reg_pc(&mut self, pc: u16) {
        self.context.reg_pc = pc;
        self.context.remaining_cycles = 0; // 重置剩余周期
        self.instruction_pending = false;
        self.irq_sampled = false;
    }

    fn reset(&mut self) {
        execute_interrupt(&mut self.context, Interrupt::Reset);
        self.total_cycles = 0;
        self.interrupt = None;
        self.instruction_pending = false;
        self.defer_interrupt = false;
        self.irq_line = false;
        self.halted = None;
        self.irq_sampled = false;
    }

    fn increase_cycles(&mut self, cycles: u32) {
        self.context.remaining_cycles += cycles;
    }

    fn trigger_interrupt(&mut self, interrupt: Interrupt) {
        if matches!(interrupt, Interrupt::Irq) {
            self.irq_line = true;
            return;
        }
        if self.interrupt.is_none() || matches!(interrupt, Interrupt::Nmi | Interrupt::Reset) {
            if matches!(interrupt, Interrupt::Nmi) {
                // The CPU polls before the final bus cycle. An edge after that poll
                // is serviced after the following instruction.
                self.defer_interrupt = self.context.remaining_cycles == 0;
            }
            self.interrupt = Some(interrupt);
        }
    }

    fn set_irq_line(&mut self, asserted: bool) {
        self.irq_line = asserted;
    }

    fn clock(&mut self) {
        if self.halted.is_some() {
            self.total_cycles = self.total_cycles.wrapping_add(1);
            return;
        }
        // 执行周期
        if self.context.remaining_cycles > 0 {
            self.context.remaining_cycles -= 1;
            if self.context.remaining_cycles == 1 && self.instruction_pending {
                self.irq_sampled = self.irq_line && self.context.reg_status & 4 == 0;
            }
            if self.context.remaining_cycles == 0 && self.instruction_pending {
                self.instruction_pending = false;
                let instruction = self.context.op.unwrap().instruction;
                execute_instruction(&mut self.context, instruction);
            }
            self.total_cycles = self.total_cycles.wrapping_add(1);
            return;
        }

        // 中断周期
        let interrupt = if self.defer_interrupt {
            self.defer_interrupt = false;
            None
        } else {
            self.interrupt
                .take()
                .or_else(|| self.irq_sampled.then_some(Interrupt::Irq))
        };
        if let Some(interrupt) = interrupt {
            self.irq_sampled = false;
            execute_interrupt(&mut self.context, interrupt);
            self.context.remaining_cycles -= 1;
            self.total_cycles = self.total_cycles.wrapping_add(1);
            return;
        }

        // 取指 && 译码
        let reg_pc = self.context.reg_pc; // 取指
        let opcode = self.context.read_bus_8bit(reg_pc); // 从总线读取指令
        if matches!(
            opcode,
            0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xb2 | 0xd2 | 0xf2
        ) {
            self.halted = Some((reg_pc, opcode));
            self.total_cycles = self.total_cycles.wrapping_add(1);
            return;
        }
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
        self.instruction_pending = true;
        self.context.remaining_cycles -= 1;
        if self.context.remaining_cycles == 1 {
            self.irq_sampled = self.irq_line && self.context.reg_status & 4 == 0;
        }
        self.total_cycles = self.total_cycles.wrapping_add(1);
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
