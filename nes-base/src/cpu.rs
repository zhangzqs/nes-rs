//! CPU 的调度接口、状态快照和中断类型，供主板连接设备及调试工具读取状态。

use std::{cell::RefCell, rc::Rc};

use crate::BusAdapter;

/// CPU 可以接收的中断或复位请求。
///
/// 请求通过 [`Cpu::trigger_interrupt`] 提交，实际响应时机由 CPU 调度器决定。
#[derive(Debug)]
pub enum Interrupt {
    /// 不可屏蔽中断事件，不受状态寄存器的 I 位限制。
    Nmi,
    /// 排队的复位请求；立即复位并解除 JAM 停机应调用 [`Cpu::reset`]。
    Reset,
    /// 可屏蔽中断请求；持续的 IRQ 电平应由 [`Cpu::set_irq_line`] 维护。
    Irq,
}

/// CPU 在某一时刻的状态快照，用于调试、执行日志和测试断言。
///
/// 快照可能位于指令执行途中，不表示 CPU 已到达下一条指令的边界。
#[derive(Debug)]
pub struct CpuState {
    /// 自最近一次 [`Cpu::reset`] 起累计的 CPU 周期，包括中断和等待周期；溢出时回绕。
    pub total_cycles: u32,
    /// 当前指令、中断或 DMA 等待尚需消耗的 CPU 周期数。
    pub remaining_cycles: u32,
    /// 累加器 A。
    pub reg_a: u8,
    /// 索引寄存器 X。
    pub reg_x: u8,
    /// 索引寄存器 Y。
    pub reg_y: u8,
    /// 栈指针 SP，表示 CPU 地址空间 `$0100..=$01FF` 内的偏移。
    pub reg_sp: u8,
    /// 程序计数器 PC；指令已译码时可能已经指向后续指令。
    pub reg_pc: u16,
    /// 处理器状态寄存器 P 的各个标志位。
    pub reg_status: CpuStatusFlags,
}

/// 状态寄存器 P 的八个位，按从低位到高位的顺序展开。
///
/// 与 `u8` 的转换仅打包或拆解位值，不修改保留位或执行任何 CPU 操作。
#[derive(Debug, Clone, Copy)]
pub struct CpuStatusFlags {
    /// C（位 0）：进位标志；减法中置位表示没有借位。
    pub carry: bool,
    /// Z（位 1）：运算结果为零。
    pub zero: bool,
    /// I（位 2）：屏蔽 IRQ；不屏蔽 NMI。
    pub interrupt_disable: bool,
    /// D（位 3）：十进制模式标志；NES CPU 保留该位，但不执行十进制算术。
    pub decimal_mode: bool,
    /// B（位 4）：BRK/PHP 压栈状态中的标记，不是持久的 CPU 中断状态。
    pub break_command: bool,
    /// 位 5：保留位，CPU 构造压栈状态时置为 1。
    pub unused: bool,
    /// V（位 6）：有符号算术溢出标志。
    pub overflow: bool,
    /// N（位 7）：负数标志，取运算结果或被测试值的最高位。
    pub negative: bool,
}

/// 将状态寄存器字节逐位拆解为标志，不规范化 B 位或保留位。
impl From<u8> for CpuStatusFlags {
    fn from(value: u8) -> Self {
        Self {
            carry: value & 0b0000_0001 != 0,
            zero: value & 0b0000_0010 != 0,
            interrupt_disable: value & 0b0000_0100 != 0,
            decimal_mode: value & 0b0000_1000 != 0,
            break_command: value & 0b0001_0000 != 0,
            unused: value & 0b0010_0000 != 0,
            overflow: value & 0b0100_0000 != 0,
            negative: value & 0b1000_0000 != 0,
        }
    }
}

/// 将各个标志按原始位序打包为状态寄存器字节。
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

/// 主板与 CPU 交互的接口，以单个 CPU 时钟周期为调度单位。
///
/// 使用前先调用 [`Cpu::attach_bus`]，再调用 [`Cpu::reset`]；运行时由主板反复
/// 调用 [`Cpu::clock`]，并独立推进 PPU/APU。该接口不保证逐拍重现所有总线访问。
pub trait Cpu {
    /// 返回 JAM 停机原因，元组为 `(触发指令的地址, 操作码)`。
    ///
    /// `None` 表示未报告停机；默认实现始终返回 `None`。JAM 后 CPU 不再执行指令，
    /// 直到调用 [`Cpu::reset`]；PPU/APU 仍可由主板继续推进。
    fn halt_reason(&self) -> Option<(u16, u8)> {
        None
    }

    /// 直接设置程序计数器，并丢弃当前待执行指令及其剩余周期，供调试或测试跳转使用。
    ///
    /// 此操作不是复位，不会重置其他寄存器、外设或解除 JAM 停机。
    fn set_reg_pc(&mut self, pc: u16);

    /// 读取 `$FFFC/$FFFD` 的复位向量，初始化 CPU 复位状态。
    ///
    /// 当前实现清除待处理中断和 JAM 停机，将总周期计数归零，并保留 7 个复位周期
    /// 供 [`Cpu::clock`] 消耗。调用前必须挂接总线；此方法不复位 RAM、PPU 或 APU。
    fn reset(&mut self);

    /// 挂接或替换 CPU 的地址总线，供取指、内存读写和中断向量读取使用。
    ///
    /// `bus` 与主板共享所有权；调用此方法本身不执行复位或推进时钟。
    fn attach_bus(&mut self, bus: Rc<RefCell<dyn BusAdapter>>);

    /// 读取当前寄存器和周期计数的快照，不执行指令或推进时钟。
    fn dump_state(&self) -> CpuState;

    /// 返回尚需消耗的 CPU 周期数；为 0 时，当前指令、中断或等待阶段已经结束。
    ///
    /// 默认从 [`Cpu::dump_state`] 读取；实现可直接读取计数器，避免构造完整快照。
    /// 返回 0 不代表 CPU 可继续取指，还需考虑 [`Cpu::halt_reason`]。
    fn remaining_cycles(&self) -> u32 {
        self.dump_state().remaining_cycles
    }

    /// 在剩余执行时间上追加 `cycles` 个 CPU 周期，用于 DMA 等待或指令附加时序。
    ///
    /// 仅增加待消耗周期，不立即执行总线操作或增加已累计的总周期数。
    fn increase_cycles(&mut self, cycles: u32);

    /// 提交中断事件或复位请求，不立即跳转到中断处理程序。
    ///
    /// 当前实现将 NMI/Reset 排队，IRQ 请求则置起 IRQ 输入电平；请求在后续时钟中
    /// 处理，IRQ 响应还受 I 位和采样时机约束。需要撤销 IRQ 时使用
    /// [`Cpu::set_irq_line`]；需要立即复位并解除 JAM 时使用 [`Cpu::reset`]。
    fn trigger_interrupt(&mut self, interrupt: Interrupt);

    /// 更新 IRQ 输入电平：`true` 表示请求有效，`false` 表示撤销请求。
    ///
    /// 是否响应由 CPU 的 I 位和中断采样时机决定。默认实现仅在 `true` 时转交
    /// [`Cpu::trigger_interrupt`]，不处理撤销；需要持续跟踪 IRQ 电平的实现应覆写此方法。
    fn set_irq_line(&mut self, asserted: bool) {
        if asserted {
            self.trigger_interrupt(Interrupt::Irq);
        }
    }

    /// 推进一个 CPU 时钟周期，处理当前指令、中断或等待阶段。
    ///
    /// 每次调用只累计一个 CPU 周期，不自动推进 PPU/APU。当前实现遇到 JAM 后仍累计
    /// 时钟周期，但不再执行指令；调用前应已挂接总线并完成 [`Cpu::reset`]。
    fn clock(&mut self);
}
