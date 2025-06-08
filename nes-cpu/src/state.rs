use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::LazyLock};

use log::debug;
use nes_base::{BusAdapter, Interrupt};

use crate::common::{AddressingMode, InstructionEnum};
use crate::opcode::Op;

// 状态标志位
#[derive(Debug, Clone, Copy)]
enum StatusFlag {
    Carry,
    Zero,
    InterruptDisable,
    DecimalMode,
    BreakCommand,
    Unused,
    Overflow,
    Negative,
}

// ctx 结构体
pub struct Context {
    // 总线的读写器
    pub bus: Option<Rc<RefCell<dyn BusAdapter>>>,

    // ctx 寄存器
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub reg_sp: u8,
    pub reg_pc: u16,
    pub reg_status: u8,

    pub remaining_cycles: u32,
    pub data_address: u16,
    pub op: Option<Op>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            bus: None,
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_sp: 0xFD, // 栈指针初始值
            reg_pc: 0xFFFC,
            reg_status: 0x24, // Unused 和 Break flags set
            remaining_cycles: 7,
            data_address: 0,
            op: None,
        }
    }
}

// 辅助方法
fn get_bit(value: u8, bit: u8) -> bool {
    (value >> bit) & 1 != 0
}

fn set_bit(value: &mut u8, bit: u8, set: bool) {
    if set {
        *value |= 1 << bit;
    } else {
        *value &= !(1 << bit);
    }
}

fn is_page_crossed(addr1: u16, addr2: u16) -> bool {
    (addr1 & 0xFF00) != (addr2 & 0xFF00)
}

// 状态寄存器操作
fn get_status_flag(status: u8, flag: StatusFlag) -> bool {
    let bit = match flag {
        StatusFlag::Carry => 0,
        StatusFlag::Zero => 1,
        StatusFlag::InterruptDisable => 2,
        StatusFlag::DecimalMode => 3,
        StatusFlag::BreakCommand => 4,
        StatusFlag::Unused => 5,
        StatusFlag::Overflow => 6,
        StatusFlag::Negative => 7,
    };
    get_bit(status, bit)
}

fn set_status_flag(status: u8, flag: StatusFlag, value: bool) -> u8 {
    let bit = match flag {
        StatusFlag::Carry => 0,
        StatusFlag::Zero => 1,
        StatusFlag::InterruptDisable => 2,
        StatusFlag::DecimalMode => 3,
        StatusFlag::BreakCommand => 4,
        StatusFlag::Unused => 5,
        StatusFlag::Overflow => 6,
        StatusFlag::Negative => 7,
    };

    let mut status = status;
    set_bit(&mut status, bit, value);
    status
}

impl Context {
    // 总线读写方法
    pub fn read_bus_8bit(&self, addr: u16) -> u8 {
        if self.bus.is_none() {
            panic!("Bus is not attached to the context");
        }
        self.bus.as_ref().unwrap().borrow().read(addr)
    }

    fn write_bus_8bit(&self, addr: u16, value: u8) {
        if self.bus.is_none() {
            panic!("Bus is not attached to the context");
        }
        self.bus.as_ref().unwrap().borrow_mut().write(addr, value);
    }

    fn read_bus_16bit(&self, addr: u16) -> u16 {
        if self.bus.is_none() {
            panic!("Bus is not attached to the context");
        }
        self.bus.as_ref().unwrap().borrow().read_u16(addr)
    }

    // 栈操作
    fn push_stack(&mut self, value: u8) {
        self.write_bus_8bit(0x0100 + self.reg_sp as u16, value);
        self.reg_sp = self.reg_sp.wrapping_sub(1);
    }

    fn pop_stack(&mut self) -> u8 {
        self.reg_sp = self.reg_sp.wrapping_add(1);
        self.read_bus_8bit(0x0100 + self.reg_sp as u16)
    }

    fn push_stack_16bit(&mut self, value: u16) {
        self.push_stack((value >> 8) as u8);
        self.push_stack(value as u8);
    }

    fn pop_stack_16bit(&mut self) -> u16 {
        let lo = self.pop_stack() as u16;
        let hi = self.pop_stack() as u16;
        (hi << 8) | lo
    }

    // 状态寄存器操作
    fn get_status_flag(&self, flag: StatusFlag) -> bool {
        get_status_flag(self.reg_status, flag)
    }

    fn set_status_flag(&mut self, flag: StatusFlag, value: bool) {
        self.reg_status = set_status_flag(self.reg_status, flag, value);
    }

    fn get_op_mode(&self) -> AddressingMode {
        self.op.unwrap().mode
    }
}

// 分支成功辅助函数
fn branch_success(ctx: &mut Context) {
    if is_page_crossed(ctx.data_address, ctx.reg_pc + 1) {
        ctx.remaining_cycles += 2;
    } else {
        ctx.remaining_cycles += 1;
    }
    ctx.reg_pc = ctx.data_address;
}

fn instruction_adc(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    let tmp = ctx.reg_a as u16 + fetched as u16 + ctx.get_status_flag(StatusFlag::Carry) as u16;

    ctx.set_status_flag(
        StatusFlag::Overflow,
        ((tmp as u8 ^ ctx.reg_a) & 0x80 != 0) && ((fetched ^ tmp as u8) & 0x80 != 0),
    );
    ctx.set_status_flag(StatusFlag::Carry, tmp > 0xFF);
    ctx.set_status_flag(StatusFlag::Zero, (tmp as u8) == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(tmp as u8, 7));

    ctx.reg_a = tmp as u8;
}

fn instruction_and(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    ctx.reg_a &= fetched;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_a == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_a, 7));
}

fn instruction_asl(ctx: &mut Context) {
    let tmp = if ctx.get_op_mode() == AddressingMode::Accumulator {
        ctx.reg_a
    } else {
        ctx.read_bus_8bit(ctx.data_address)
    };
    ctx.set_status_flag(StatusFlag::Carry, get_bit(tmp, 7));
    let tmp = tmp << 1;
    if ctx.get_op_mode() == AddressingMode::Accumulator {
        ctx.reg_a = tmp;
    } else {
        ctx.write_bus_8bit(ctx.data_address, tmp);
    }
    ctx.set_status_flag(StatusFlag::Zero, tmp == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(tmp, 7));
}

fn instruction_bit(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    let test = fetched & ctx.reg_a;
    ctx.set_status_flag(StatusFlag::Zero, test == 0);
    ctx.set_status_flag(StatusFlag::Overflow, get_bit(fetched, 6));
    ctx.set_status_flag(StatusFlag::Negative, get_bit(fetched, 7));
}

fn instruction_bcc(ctx: &mut Context) {
    if !ctx.get_status_flag(StatusFlag::Carry) {
        branch_success(ctx);
    }
}

fn instruction_bcs(ctx: &mut Context) {
    if ctx.get_status_flag(StatusFlag::Carry) {
        branch_success(ctx);
    }
}

fn instruction_beq(ctx: &mut Context) {
    if ctx.get_status_flag(StatusFlag::Zero) {
        branch_success(ctx);
    }
}

fn instruction_bmi(ctx: &mut Context) {
    if ctx.get_status_flag(StatusFlag::Negative) {
        branch_success(ctx);
    }
}

fn instruction_bne(ctx: &mut Context) {
    if !ctx.get_status_flag(StatusFlag::Zero) {
        branch_success(ctx);
    }
}

fn instruction_bpl(ctx: &mut Context) {
    if !ctx.get_status_flag(StatusFlag::Negative) {
        branch_success(ctx);
    }
}

fn instruction_bvc(ctx: &mut Context) {
    if !ctx.get_status_flag(StatusFlag::Overflow) {
        branch_success(ctx);
    }
}

fn instruction_bvs(ctx: &mut Context) {
    if ctx.get_status_flag(StatusFlag::Overflow) {
        branch_success(ctx);
    }
}

fn instruction_brk(ctx: &mut Context) {
    ctx.push_stack_16bit(ctx.reg_pc + 1);
    ctx.push_stack(ctx.reg_status);
    ctx.set_status_flag(StatusFlag::InterruptDisable, true);
    ctx.set_status_flag(StatusFlag::BreakCommand, true);
    ctx.reg_pc = ctx.read_bus_16bit(0xFFFE);
}

fn instruction_clc(ctx: &mut Context) {
    ctx.set_status_flag(StatusFlag::Carry, false);
}

fn instruction_cld(ctx: &mut Context) {
    ctx.set_status_flag(StatusFlag::DecimalMode, false);
}

fn instruction_cli(ctx: &mut Context) {
    ctx.set_status_flag(StatusFlag::InterruptDisable, false);
}

fn instruction_clv(ctx: &mut Context) {
    ctx.set_status_flag(StatusFlag::Overflow, false);
}

fn instruction_cmp(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    let tmp = ctx.reg_a.wrapping_sub(fetched);
    ctx.set_status_flag(StatusFlag::Carry, ctx.reg_a >= fetched);
    ctx.set_status_flag(StatusFlag::Zero, tmp == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(tmp, 7));
}

fn instruction_cpx(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    let tmp = ctx.reg_x.wrapping_sub(fetched);
    ctx.set_status_flag(StatusFlag::Carry, ctx.reg_x >= fetched);
    ctx.set_status_flag(StatusFlag::Zero, tmp == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(tmp, 7));
}

fn instruction_cpy(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    let tmp = ctx.reg_y.wrapping_sub(fetched);
    ctx.set_status_flag(StatusFlag::Carry, ctx.reg_y >= fetched);
    ctx.set_status_flag(StatusFlag::Zero, tmp == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(tmp, 7));
}

fn instruction_dec(ctx: &mut Context) {
    let mut fetched = ctx.read_bus_8bit(ctx.data_address);
    fetched = fetched.wrapping_sub(1);
    ctx.write_bus_8bit(ctx.data_address, fetched);
    ctx.set_status_flag(StatusFlag::Zero, fetched == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(fetched, 7));
}

fn instruction_dex(ctx: &mut Context) {
    ctx.reg_x = ctx.reg_x.wrapping_sub(1);
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_x == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_x, 7));
}

fn instruction_dey(ctx: &mut Context) {
    ctx.reg_y = ctx.reg_y.wrapping_sub(1);
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_y == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_y, 7));
}

fn instruction_eor(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    ctx.reg_a ^= fetched;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_a == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_a, 7));
}

fn instruction_inc(ctx: &mut Context) {
    let mut fetched = ctx.read_bus_8bit(ctx.data_address);
    fetched = fetched.wrapping_add(1);
    ctx.write_bus_8bit(ctx.data_address, fetched);
    ctx.set_status_flag(StatusFlag::Zero, fetched == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(fetched, 7));
}

fn instruction_inx(ctx: &mut Context) {
    ctx.reg_x = ctx.reg_x.wrapping_add(1);
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_x == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_x, 7));
}

fn instruction_iny(ctx: &mut Context) {
    ctx.reg_y = ctx.reg_y.wrapping_add(1);
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_y == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_y, 7));
}

fn instruction_jmp(ctx: &mut Context) {
    ctx.reg_pc = ctx.data_address;
}

fn instruction_jsr(ctx: &mut Context) {
    ctx.push_stack_16bit(ctx.reg_pc - 1);
    ctx.reg_pc = ctx.data_address;
}

fn instruction_lda(ctx: &mut Context) {
    debug!("LDA: mode={:?}", ctx.get_op_mode());
    debug!("LDA: data_address={:#04x}", ctx.data_address);
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    debug!("LDA: fetched={:#04x}", fetched);
    ctx.reg_a = fetched;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_a == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_a, 7));
}

fn instruction_ldx(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    ctx.reg_x = fetched;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_x == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_x, 7));
}

fn instruction_ldy(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    ctx.reg_y = fetched;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_y == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_y, 7));
}

/// 逻辑右移指令，将目标操作数右移，并将最低位移入进位标志位。
fn instruction_lsr(ctx: &mut Context) {
    debug!("LSR: mode={:?}", ctx.get_op_mode());
    let tmp = if ctx.get_op_mode() == AddressingMode::Accumulator {
        ctx.reg_a
    } else {
        ctx.read_bus_8bit(ctx.data_address)
    };
    debug!("LSR: tmp={:#04x}", tmp);
    ctx.set_status_flag(StatusFlag::Carry, get_bit(tmp, 0));
    let tmp = tmp >> 1;
    if ctx.get_op_mode() == AddressingMode::Accumulator {
        ctx.reg_a = tmp;
    } else {
        ctx.write_bus_8bit(ctx.data_address, tmp);
    }
    ctx.set_status_flag(StatusFlag::Zero, tmp == 0);
    ctx.set_status_flag(StatusFlag::Negative, false);
}

fn instruction_ora(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    ctx.reg_a |= fetched;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_a == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_a, 7));
}

fn instruction_pha(ctx: &mut Context) {
    ctx.push_stack(ctx.reg_a);
}

fn instruction_php(ctx: &mut Context) {
    ctx.push_stack(ctx.reg_status | 0x30);
}

fn instruction_pla(ctx: &mut Context) {
    ctx.reg_a = ctx.pop_stack();
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_a == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_a, 7));
}

fn instruction_plp(ctx: &mut Context) {
    let status = ctx.pop_stack();
    let status = set_status_flag(
        status,
        StatusFlag::BreakCommand,
        ctx.get_status_flag(StatusFlag::BreakCommand),
    );
    let status = set_status_flag(
        status,
        StatusFlag::Unused,
        ctx.get_status_flag(StatusFlag::Unused),
    );
    ctx.reg_status = status;
}

fn instruction_rol(ctx: &mut Context) {
    let tmp = if ctx.get_op_mode() == AddressingMode::Accumulator {
        ctx.reg_a
    } else {
        ctx.read_bus_8bit(ctx.data_address)
    };
    let old_carry = ctx.get_status_flag(StatusFlag::Carry);
    ctx.set_status_flag(StatusFlag::Carry, get_bit(tmp, 7));
    let tmp = (tmp << 1) | old_carry as u8;
    if ctx.get_op_mode() == AddressingMode::Accumulator {
        ctx.reg_a = tmp;
        ctx.set_status_flag(StatusFlag::Zero, ctx.reg_a == 0);
    } else {
        ctx.write_bus_8bit(ctx.data_address, tmp);
    }
    ctx.set_status_flag(StatusFlag::Negative, get_bit(tmp, 7));
}

fn instruction_ror(ctx: &mut Context) {
    let tmp = if ctx.get_op_mode() == AddressingMode::Accumulator {
        ctx.reg_a
    } else {
        ctx.read_bus_8bit(ctx.data_address)
    };
    let old_carry = ctx.get_status_flag(StatusFlag::Carry);
    ctx.set_status_flag(StatusFlag::Carry, get_bit(tmp, 0));
    let tmp = (tmp >> 1) | ((old_carry as u8) << 7);
    if ctx.get_op_mode() == AddressingMode::Accumulator {
        ctx.reg_a = tmp;
        ctx.set_status_flag(StatusFlag::Zero, ctx.reg_a == 0);
    } else {
        ctx.write_bus_8bit(ctx.data_address, tmp);
    }
    ctx.set_status_flag(StatusFlag::Negative, get_bit(tmp, 7));
}

fn instruction_rti(ctx: &mut Context) {
    let status = ctx.pop_stack();
    let status = set_status_flag(
        status,
        StatusFlag::BreakCommand,
        ctx.get_status_flag(StatusFlag::BreakCommand),
    );
    let status = set_status_flag(
        status,
        StatusFlag::Unused,
        ctx.get_status_flag(StatusFlag::Unused),
    );
    ctx.reg_status = status;
    ctx.reg_pc = ctx.pop_stack_16bit();
}

fn instruction_rts(ctx: &mut Context) {
    ctx.reg_pc = ctx.pop_stack_16bit() + 1;
}

fn instruction_sbc(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    let tmp =
        ctx.reg_a as i16 - fetched as i16 - (1 - ctx.get_status_flag(StatusFlag::Carry) as i16);
    ctx.set_status_flag(
        StatusFlag::Overflow,
        ((tmp as u8 ^ ctx.reg_a) & 0x80 != 0) && ((ctx.reg_a ^ fetched) & 0x80 != 0),
    );
    ctx.set_status_flag(StatusFlag::Carry, tmp >= 0);
    ctx.set_status_flag(StatusFlag::Zero, (tmp as u8) == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(tmp as u8, 7));
    ctx.reg_a = tmp as u8;
}

fn instruction_sec(ctx: &mut Context) {
    ctx.set_status_flag(StatusFlag::Carry, true);
}

fn instruction_sed(ctx: &mut Context) {
    ctx.set_status_flag(StatusFlag::DecimalMode, true);
}

fn instruction_sei(ctx: &mut Context) {
    ctx.set_status_flag(StatusFlag::InterruptDisable, true);
}

fn instruction_sta(ctx: &mut Context) {
    ctx.write_bus_8bit(ctx.data_address, ctx.reg_a);
}

fn instruction_stx(ctx: &mut Context) {
    ctx.write_bus_8bit(ctx.data_address, ctx.reg_x);
}

fn instruction_sty(ctx: &mut Context) {
    ctx.write_bus_8bit(ctx.data_address, ctx.reg_y);
}

fn instruction_tax(ctx: &mut Context) {
    ctx.reg_x = ctx.reg_a;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_x == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_x, 7));
}

fn instruction_tay(ctx: &mut Context) {
    ctx.reg_y = ctx.reg_a;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_y == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_y, 7));
}

fn instruction_tsx(ctx: &mut Context) {
    ctx.reg_x = ctx.reg_sp;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_x == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_x, 7));
}

fn instruction_txa(ctx: &mut Context) {
    ctx.reg_a = ctx.reg_x;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_a == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_a, 7));
}

fn instruction_txs(ctx: &mut Context) {
    ctx.reg_sp = ctx.reg_x;
}

fn instruction_tya(ctx: &mut Context) {
    ctx.reg_a = ctx.reg_y;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_a == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_a, 7));
}

// 非法/复合指令
fn instruction_alr(ctx: &mut Context) {
    instruction_and(ctx);
    instruction_lsr(ctx);
}

fn instruction_anc(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    ctx.reg_a &= fetched;
    ctx.set_status_flag(StatusFlag::Carry, get_bit(ctx.reg_a, 7));
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_a == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_a, 7));
}

fn instruction_arr(ctx: &mut Context) {
    instruction_and(ctx);
    instruction_ror(ctx);
}

fn instruction_axs(ctx: &mut Context) {
    ctx.reg_x &= ctx.reg_a;
    ctx.set_status_flag(StatusFlag::Carry, false);
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_x == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_x, 7));
}

fn instruction_lax(ctx: &mut Context) {
    let fetched = ctx.read_bus_8bit(ctx.data_address);
    ctx.reg_x = fetched;
    ctx.reg_a = fetched;
    ctx.set_status_flag(StatusFlag::Zero, ctx.reg_a == 0);
    ctx.set_status_flag(StatusFlag::Negative, get_bit(ctx.reg_a, 7));
}

fn instruction_sax(ctx: &mut Context) {
    ctx.write_bus_8bit(ctx.data_address, ctx.reg_x & ctx.reg_a);
}

fn instruction_dcp(ctx: &mut Context) {
    instruction_dec(ctx);
    instruction_cmp(ctx);
}

fn instruction_isc(ctx: &mut Context) {
    instruction_inc(ctx);
    instruction_sbc(ctx);
}

fn instruction_rla(ctx: &mut Context) {
    instruction_rol(ctx);
    instruction_and(ctx);
}

fn instruction_rra(ctx: &mut Context) {
    instruction_ror(ctx);
    instruction_adc(ctx);
}

fn instruction_slo(ctx: &mut Context) {
    instruction_asl(ctx);
    instruction_ora(ctx);
}

fn instruction_sre(ctx: &mut Context) {
    instruction_lsr(ctx);
    instruction_eor(ctx);
}

// NOP/IGN/SKB
fn instruction_nop(_: &mut Context) {}
fn instruction_skb(_: &mut Context) {}
fn instruction_ign(_: &mut Context) {}

struct InstructionManager {
    instructions: HashMap<InstructionEnum, fn(&mut Context)>,
}

impl InstructionManager {
    fn new() -> Self {
        let mut m: HashMap<InstructionEnum, fn(&mut Context)> = HashMap::new();

        m.insert(InstructionEnum::ADC, instruction_adc);
        m.insert(InstructionEnum::AND, instruction_and);
        m.insert(InstructionEnum::ASL, instruction_asl);
        m.insert(InstructionEnum::BIT, instruction_bit);
        m.insert(InstructionEnum::BCC, instruction_bcc);
        m.insert(InstructionEnum::BCS, instruction_bcs);
        m.insert(InstructionEnum::BEQ, instruction_beq);
        m.insert(InstructionEnum::BMI, instruction_bmi);
        m.insert(InstructionEnum::BNE, instruction_bne);
        m.insert(InstructionEnum::BPL, instruction_bpl);
        m.insert(InstructionEnum::BVC, instruction_bvc);
        m.insert(InstructionEnum::BVS, instruction_bvs);
        m.insert(InstructionEnum::BRK, instruction_brk);
        m.insert(InstructionEnum::CLC, instruction_clc);
        m.insert(InstructionEnum::CLD, instruction_cld);
        m.insert(InstructionEnum::CLI, instruction_cli);
        m.insert(InstructionEnum::CLV, instruction_clv);
        m.insert(InstructionEnum::CMP, instruction_cmp);
        m.insert(InstructionEnum::CPX, instruction_cpx);
        m.insert(InstructionEnum::CPY, instruction_cpy);
        m.insert(InstructionEnum::DEC, instruction_dec);
        m.insert(InstructionEnum::DEX, instruction_dex);
        m.insert(InstructionEnum::DEY, instruction_dey);
        m.insert(InstructionEnum::EOR, instruction_eor);
        m.insert(InstructionEnum::INC, instruction_inc);
        m.insert(InstructionEnum::INX, instruction_inx);
        m.insert(InstructionEnum::INY, instruction_iny);
        m.insert(InstructionEnum::JMP, instruction_jmp);
        m.insert(InstructionEnum::JSR, instruction_jsr);
        m.insert(InstructionEnum::LDA, instruction_lda);
        m.insert(InstructionEnum::LDX, instruction_ldx);
        m.insert(InstructionEnum::LDY, instruction_ldy);
        m.insert(InstructionEnum::LSR, instruction_lsr);
        m.insert(InstructionEnum::ORA, instruction_ora);
        m.insert(InstructionEnum::PHA, instruction_pha);
        m.insert(InstructionEnum::PHP, instruction_php);
        m.insert(InstructionEnum::PLA, instruction_pla);
        m.insert(InstructionEnum::PLP, instruction_plp);
        m.insert(InstructionEnum::ROL, instruction_rol);
        m.insert(InstructionEnum::ROR, instruction_ror);
        m.insert(InstructionEnum::RTI, instruction_rti);
        m.insert(InstructionEnum::RTS, instruction_rts);
        m.insert(InstructionEnum::SBC, instruction_sbc);
        m.insert(InstructionEnum::SEC, instruction_sec);
        m.insert(InstructionEnum::SED, instruction_sed);
        m.insert(InstructionEnum::SEI, instruction_sei);
        m.insert(InstructionEnum::STA, instruction_sta);
        m.insert(InstructionEnum::STX, instruction_stx);
        m.insert(InstructionEnum::STY, instruction_sty);
        m.insert(InstructionEnum::TAX, instruction_tax);
        m.insert(InstructionEnum::TAY, instruction_tay);
        m.insert(InstructionEnum::TSX, instruction_tsx);
        m.insert(InstructionEnum::TXA, instruction_txa);
        m.insert(InstructionEnum::TXS, instruction_txs);
        m.insert(InstructionEnum::TYA, instruction_tya);
        m.insert(InstructionEnum::ALR, instruction_alr);
        m.insert(InstructionEnum::ANC, instruction_anc);
        m.insert(InstructionEnum::ARR, instruction_arr);
        m.insert(InstructionEnum::AXS, instruction_axs);
        m.insert(InstructionEnum::LAX, instruction_lax);
        m.insert(InstructionEnum::SAX, instruction_sax);
        m.insert(InstructionEnum::DCP, instruction_dcp);
        m.insert(InstructionEnum::ISC, instruction_isc);
        m.insert(InstructionEnum::RLA, instruction_rla);
        m.insert(InstructionEnum::RRA, instruction_rra);
        m.insert(InstructionEnum::SLO, instruction_slo);
        m.insert(InstructionEnum::SRE, instruction_sre);
        m.insert(InstructionEnum::NOP, instruction_nop);
        m.insert(InstructionEnum::SKB, instruction_skb);
        m.insert(InstructionEnum::IGN, instruction_ign);

        Self { instructions: m }
    }
}

static INSTRUCTION_MANAGER: LazyLock<InstructionManager> = LazyLock::new(InstructionManager::new);

pub fn execute_instruction(ctx: &mut Context, instruction: InstructionEnum) {
    if let Some(&func) = INSTRUCTION_MANAGER.instructions.get(&instruction) {
        func(ctx);
    } else {
        panic!("Unknown instruction: {:?}", instruction);
    }
}

fn interrupt_nmi(ctx: &mut Context) {
    ctx.push_stack_16bit(ctx.reg_pc);
    ctx.push_stack(ctx.reg_status | 0x30);
    ctx.set_status_flag(StatusFlag::InterruptDisable, true);
    ctx.reg_pc = ctx.read_bus_16bit(0xFFFA);

    ctx.remaining_cycles = 7;
}

fn interrupt_reset(ctx: &mut Context) {
    ctx.reg_sp = 0xFD;
    ctx.reg_status = 0x24; // Unused and Break flags set
    ctx.reg_pc = ctx.read_bus_16bit(0xFFFC);

    ctx.remaining_cycles = 7;
}

fn interrupt_irq(ctx: &mut Context) {
    if ctx.get_status_flag(StatusFlag::InterruptDisable) {
        return;
    }
    ctx.push_stack_16bit(ctx.reg_pc);
    ctx.push_stack(ctx.reg_status | 0x30);
    ctx.set_status_flag(StatusFlag::InterruptDisable, true);
    ctx.reg_pc = ctx.read_bus_16bit(0xFFFE);

    ctx.remaining_cycles = 7;
}

pub fn execute_interrupt(ctx: &mut Context, signal: Interrupt) {
    match signal {
        Interrupt::Nmi => interrupt_nmi(ctx),
        Interrupt::Reset => interrupt_reset(ctx),
        Interrupt::Irq => interrupt_irq(ctx),
    }
}

pub struct AddressingModeResult {
    pub address: u16,
    pub page_crossed: bool,
    pub pc_increment: u16,
}

// Addressing mode functions
// see: https://wiki.nesdev.com/w/index.php/CPU_addressing_modes
pub fn get_data_address(ctx: &Context) -> AddressingModeResult {
    let read_bus_16bit_uncross_page = |addr: u16| {
        let next_addr_high = addr & 0xFF00;
        let next_addr_low = (addr + 1) & 0xFF;
        let next_addr = next_addr_high | next_addr_low;
        (ctx.read_bus_8bit(next_addr) as u16) << 8 | ctx.read_bus_8bit(addr) as u16
    };

    match ctx.get_op_mode() {
        AddressingMode::Immediate => AddressingModeResult {
            address: ctx.reg_pc + 1,
            page_crossed: false,
            pc_increment: 2,
        },
        AddressingMode::ZeroPage => {
            let addr = ctx.read_bus_8bit(ctx.reg_pc + 1) as u16;
            AddressingModeResult {
                address: addr,
                page_crossed: false,
                pc_increment: 2,
            }
        }
        AddressingMode::ZeroPageX => {
            let addr = ctx.read_bus_8bit(ctx.reg_pc + 1).wrapping_add(ctx.reg_x) as u16;
            AddressingModeResult {
                address: addr,
                page_crossed: false,
                pc_increment: 2,
            }
        }
        AddressingMode::ZeroPageY => {
            let addr = ctx.read_bus_8bit(ctx.reg_pc + 1).wrapping_add(ctx.reg_y) as u16;
            AddressingModeResult {
                address: addr,
                page_crossed: false,
                pc_increment: 2,
            }
        }
        AddressingMode::Absolute => {
            let addr = ctx.read_bus_16bit(ctx.reg_pc + 1);
            AddressingModeResult {
                address: addr,
                page_crossed: false,
                pc_increment: 3,
            }
        }
        AddressingMode::AbsoluteX => {
            let base_addr = ctx.read_bus_16bit(ctx.reg_pc + 1);
            let addr = base_addr.wrapping_add(ctx.reg_x as u16);
            AddressingModeResult {
                address: addr,
                page_crossed: is_page_crossed(base_addr, addr),
                pc_increment: 3,
            }
        }
        AddressingMode::AbsoluteY => {
            let base_addr = ctx.read_bus_16bit(ctx.reg_pc + 1);
            let addr = base_addr.wrapping_add(ctx.reg_y as u16);
            AddressingModeResult {
                address: addr,
                page_crossed: is_page_crossed(base_addr, addr),
                pc_increment: 3,
            }
        }
        AddressingMode::IndexedIndirect => {
            let base_addr = ctx.read_bus_8bit(ctx.reg_pc + 1);
            let addr = read_bus_16bit_uncross_page(base_addr.wrapping_add(ctx.reg_x) as u16);
            AddressingModeResult {
                address: addr,
                page_crossed: false,
                pc_increment: 2,
            }
        }
        AddressingMode::IndirectIndexed => {
            let base_addr = ctx.read_bus_8bit(ctx.reg_pc + 1) as u16;
            let addr = read_bus_16bit_uncross_page(base_addr).wrapping_add(ctx.reg_y as u16);
            AddressingModeResult {
                address: addr,
                page_crossed: is_page_crossed(addr, addr.wrapping_sub(ctx.reg_y as u16)),
                pc_increment: 2,
            }
        }
        AddressingMode::Accumulator => {
            // 累加器模式没有地址，只是操作累加器
            AddressingModeResult {
                address: 0,
                page_crossed: false,
                pc_increment: 1,
            }
        }
        AddressingMode::Relative => {
            let offset = ctx.read_bus_8bit(ctx.reg_pc + 1) as i8;
            let addr = ctx.reg_pc.wrapping_add(2).wrapping_add(offset as u16);
            AddressingModeResult {
                address: addr,
                page_crossed: false,
                pc_increment: 2,
            }
        }
        AddressingMode::Implied => {
            // 隐含寻址模式没有地址，只是执行指令
            AddressingModeResult {
                address: 0,
                page_crossed: false,
                pc_increment: 1,
            }
        }
        AddressingMode::Indirect => {
            let base_addr = ctx.read_bus_16bit(ctx.reg_pc + 1);
            let addr = read_bus_16bit_uncross_page(base_addr);
            AddressingModeResult {
                address: addr,
                page_crossed: false,
                pc_increment: 3,
            }
        }
    }
}
