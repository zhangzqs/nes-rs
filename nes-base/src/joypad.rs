use std::cell::RefCell;

use crate::{addressable::*, flag_reg};

flag_reg!(
    JoypadButton,
    button_a,
    button_b,
    select,
    start,
    up,
    down,
    left,
    right
);

/// 游戏机有两个手柄,分别映射到0x4016与0x4017两个cpu地址空间
/// 同一个寄存器是可读可写的,通过读取按钮状态(1按下, 0释放),
/// 来上传按钮状态,为了获取所有按钮状态,cpu必须读取控制寄存器8次
/// Button的上报顺序如下
/// A -> B -> Select -> Start -> Up -> Down -> Left -> Right
/// 在上报完right的 状态后,控制器将返回连续的1用于后续的读取,直到strobe发生变化
/// CPU可通过向寄存器写入一个字节来改变控制器模式,只有第一位重要

/// 手柄控制器有两种模式运行
/// + strobe on   : 此时指针重置到按钮 A
/// + strobe off  : 循环上报按钮状态

pub struct Joypad {
    button: JoypadButton,
    strobe: bool,
    button_pointer: RefCell<u8>,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            button: JoypadButton::from(0),
            strobe: false,
            button_pointer: RefCell::new(0),
        }
    }
}

impl Readable for Joypad {
    fn read(&self, _: u16) -> u8 {
        // 在上报完right的状态后,控制器将返回连续的1用于后续的读取
        let mut btn_ptr_ref = self.button_pointer.borrow_mut();
        let btn_ptr = btn_ptr_ref.clone();
        if btn_ptr > 7 {
            return 1;
        }
        let button_bits: u8 = self.button.into();
        let response = button_bits >> btn_ptr & 1;
        if !self.strobe && btn_ptr <= 7 {
            *btn_ptr_ref = btn_ptr + 1;
        }
        response
    }
}
impl Writable for Joypad {
    // CPU可通过向寄存器写入一个字节来改变控制器模式,只有第一位重要
    fn write(&mut self, _: u16, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            let mut btn_ptr_ref = self.button_pointer.borrow_mut();
            *btn_ptr_ref = 0;
        }
    }
}
impl Addressable for Joypad {}