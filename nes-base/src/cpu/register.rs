use super::status::StatusFlagRegister;

pub struct Register {
    /// 寄存器A
    pub a: u8,
    /// 寄存器X
    pub x: u8,
    /// 寄存器Y
    pub y: u8,
    /// 标志位寄存器
    pub status: StatusFlagRegister,
    /// 程序计数器(program counter)
    pub pc: u16,
    /// 栈指针寄存器(stack pointer)
    pub sp: u8,
}

const STACK_RESET: u8 = 0xFD;

impl Default for Register {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            status: Default::default(),
            pc: 0,
            sp: STACK_RESET,
        }
    }
}