use state::{Context, execute_instruction, execute_interrupt, get_data_address};

mod common;
mod opcode;
mod state;

struct CPU {
    context: state::Context,
    interrupt: Option<state::Interrupt>,
}

impl CPU {
    fn send_interrupt(&mut self, interrupt: state::Interrupt) {
        if self.interrupt.is_none() {
            self.interrupt = Some(interrupt);
        }
    }

    fn clock(&mut self) {
        // 执行周期
        if self.context.remaining_cycles > 0 {
            self.context.remaining_cycles -= 1;
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
        self.context.remaining_cycles = op.cycles;

        // 寻址
        let result = get_data_address(&self.context);
        self.context.data_address = result.address;
        self.context.increase_pc(result.pc_increment);
        if result.page_crossed && op.increase_cycle_when_cross_page {
            self.context.remaining_cycles += 1;
        }

        // 执行指令
        execute_instruction(&mut self.context, op.instruction);
    }
}
