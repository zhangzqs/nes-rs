mod opcode;
mod register;
mod status;

use crate::addressable::Addressable;
use register::Register;
use status::StatusFlagRegister;

#[derive(Debug, Copy, Clone)]
pub enum AddressingMode {
    /// 立即数寻址(操作码，操作数)
    Immediate,
    /// 零页寻址(操作码，零页地址(零页即0x00~0xFF))
    ZeroPage,
    /// 零页X寻址(操作码，零页基地址)
    ZeroPageX,
    /// 零页Y寻址(操作码，零页基地址)
    ZeroPageY,
    /// 绝对寻址(操作码，操作数地址低字节，操作数地址高字节)
    Absolute,
    /// 绝对X寻址(操作码，基地址低字节，基地址高字节)
    AbsoluteX,
    /// 绝对Y寻址(操作码，基地址低字节，基地址高字节)
    AbsoluteY,
    /// X间接寻址(操作码，零页基地址)
    /// *(X+base) | *(X+base+1) << 8
    IndirectX,
    /// Y间接寻址(操作码，零页间接地址)
    /// *(base) | *(base+1) << 8 + Y
    IndirectY,
    /// 无效寻址
    NoneAddressing,
}

pub struct CPU {
    pub register: Register,
    pub bus: Box<dyn Addressable>,
}
/// 触发CPU外部中断
impl CPU {
    /// 启动时或按下游戏机的RST按键时触发
    pub fn reset(&mut self) {
        self.register = Register::default();
        self.register.pc = self.read_u16(0xFFFC);
    }
    /// CPU的irq引脚触发
    pub fn irq(&mut self) {}
}

impl CPU {
    pub fn run_with_callback<F: FnMut(&mut CPU)>(&mut self, mut callback: F) {
        while self.run_one_instruction() {
            callback(self);
        }
    }

    /// 返回值为false表示程序结束
    fn run_one_instruction(&mut self) -> bool {
        use opcode::get_opcode_by_code;
        let code = self.read(self.register.pc);
        self.register.pc += 1;
        let old_pc = self.register.pc;
        let opcode =
            get_opcode_by_code(code).expect(&format!("OpCode {:x} is not recognized", code));
        let mode = &opcode.mode;
        match code {
            0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => self.lda(mode),
            0xAA => self.tax(),
            0xa8 => self.tay(),
            0xba => self.tsx(),
            0x8a => self.txa(),
            0x9a => self.txs(),
            0x98 => self.tya(),
            0xe8 => self.inx(),
            0xd8 => self.cld(),
            0x58 => self.cli(),
            0xb8 => self.clv(),
            0x18 => self.clc(),
            0x38 => self.sec(),
            0x78 => self.sei(),
            0xf8 => self.sed(),
            0x48 => self.pha(),
            0x68 => self.pla(),
            0x08 => self.php(),
            0x28 => self.plp(),
            0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => self.adc(mode),
            0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 => self.sbc(mode),
            0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => self.and(mode),
            0x49 | 0x45 | 0x55 | 0x4d | 0x5d | 0x59 | 0x41 | 0x51 => self.eor(mode),
            0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => self.ora(mode),
            0x4a => self.lsr_reg_a(),
            0x46 | 0x56 | 0x4e | 0x5e => self.lsr_memory(mode),
            0x0a => self.asl_reg_a(),
            0x06 | 0x16 | 0x0e | 0x1e => self.asl_memory(mode),
            0x2a => self.rol_reg_a(),
            0x26 | 0x36 | 0x2e | 0x3e => self.rol_memory(mode),
            0x6a => self.ror_reg_a(),
            0x66 | 0x76 | 0x6e | 0x7e => self.ror_memory(mode),
            0xe6 | 0xf6 | 0xee | 0xfe => self.inc(mode),
            0xc8 => self.iny(),
            0xc6 | 0xd6 | 0xce | 0xde => self.dec(mode),
            0xca => self.dex(),
            0x88 => self.dey(),
            0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1 => self.cmp(mode),
            0xc0 | 0xc4 | 0xcc => self.cpy(mode),
            0xe0 | 0xe4 | 0xec => self.cpx(mode),
            0x4c => self.jmp_absolute(),
            0x6c => self.jmp_indirect(),
            0x20 => self.jsr(),
            0x60 => self.rts(),
            0x40 => self.rti(),
            0xd0 => self.bne(),
            0x70 => self.bvs(),
            0x50 => self.bvc(),
            0x10 => self.bpl(),
            0x30 => self.bmi(),
            0xf0 => self.beq(),
            0xb0 => self.bcs(),
            0x90 => self.bcc(),
            0x24 | 0x2c => self.bit(mode),
            0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => self.sta(mode),
            0x86 | 0x96 | 0x8e => self.stx(mode),
            0x84 | 0x94 | 0x8c => self.sty(mode),
            0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => self.ldx(mode),
            0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => self.ldy(mode),
            0xea => {} // NOP
            0x00 => return false,

            /* unofficial */
            0xc7 | 0xd7 | 0xCF | 0xdF | 0xdb | 0xd3 | 0xc3 => self.dcp(mode),
            0x27 | 0x37 | 0x2F | 0x3F | 0x3b | 0x33 | 0x23 => self.rla(mode),
            0x07 | 0x17 | 0x0F | 0x1f | 0x1b | 0x03 | 0x13 => self.slo(mode),
            0x47 | 0x57 | 0x4F | 0x5f | 0x5b | 0x43 | 0x53 => self.sre(mode),
            0x80 | 0x82 | 0x89 | 0xc2 | 0xe2 => self.skb(mode),
            0xCB => self.axs(mode),
            0x6B => self.arr(mode),
            0xeb => self.unofficial_sbc(mode),
            0x0b | 0x2b => self.anc(mode),
            0x4b => self.alr(mode),
            0x04 | 0x44 | 0x64 | 0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 | 0x0c | 0x1c | 0x3c
            | 0x5c | 0x7c | 0xdc | 0xfc => self.nop_read(mode),
            0x67 | 0x77 | 0x6f | 0x7f | 0x7b | 0x63 | 0x73 => self.rra(mode),
            0xe7 | 0xf7 | 0xef | 0xff | 0xfb | 0xe3 | 0xf3 => self.lsb(mode),
            0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xb2 | 0xd2 | 0xf2 => {
                self.unofficial_nop(mode)
            }
            0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => self.unofficial_nop(mode),
            0xa7 | 0xb7 | 0xaf | 0xbf | 0xa3 | 0xb3 => self.lax(mode),
            0x87 | 0x97 | 0x8f | 0x83 => self.sax(mode),
            0xab => self.lxa(mode),
            0x8b => self.xaa(mode),
            0xbb => self.las(mode),
            0x9b => self.tas(mode),
            0x93 => self.ahx_indirect_y(mode),
            0x9f => self.ahx_absolute_y(mode),
            0x9e => self.shx(mode),
            0x9c => self.shy(mode),
        }
        // 没有执行跳转指令
        if old_pc == self.register.pc {
            self.register.pc += (opcode.length - 1) as u16;
        }
        true
    }
}

/// 非官方指令
impl CPU {
    fn dcp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.read(addr);
        data = data.wrapping_sub(1);
        self.write(addr, data);
        // self._update_zero_and_negative_flags(data);
        if data <= self.register.a {
            self.register.status.carry = true;
        }

        self.update_zero_and_negative_flags(self.register.a.wrapping_sub(data));
    }
    fn rla(&mut self, mode: &AddressingMode) {
        self.rol_memory(mode);
        let data = self.get_operand(mode);
        self.and_with_register_a(data);
    }
    fn slo(&mut self, mode: &AddressingMode) {
        self.rol_memory(mode);
        let data = self.get_operand(mode);
        self.or_with_register_a(data);
    }
    fn sre(&mut self, mode: &AddressingMode) {
        self.rol_memory(mode);
        let data = self.get_operand(mode);
        self.xor_with_register_a(data);
    }
    fn skb(&mut self, mode: &AddressingMode) {

        /* 2 byte NOP (immidiate ) */
        // todo: might be worth doing the read
    }
    fn axs(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.read(addr);
        let x_and_a = self.register.x & self.register.a;
        let result = x_and_a.wrapping_sub(data);

        if data <= x_and_a {
            self.register.status.carry = true;
        }
        self.update_zero_and_negative_flags(result);

        self.register.x = result;
    }
    fn arr(&mut self, mode: &AddressingMode) {
        let data = self.get_operand(mode);
        self.and_with_register_a(data);
        self.ror_reg_a();
        //todo: registers
        let result = self.register.a;
        let bit_5 = (result >> 5) & 1;
        let bit_6 = (result >> 6) & 1;
        self.register.status.carry = bit_6 == 1;
        self.register.status.overflow = bit_5 ^ bit_6 == 1;

        self.update_zero_and_negative_flags(result);
    }
    fn unofficial_sbc(&mut self, mode: &AddressingMode) {
        let data = self.get_operand(mode);
        self.sub_from_register_a(data);
    }

    fn anc(&mut self, mode: &AddressingMode) {
        let data = self.get_operand(mode);
        self.and_with_register_a(data);
        self.register.status.carry = self.register.status.negative;
    }
    fn alr(&mut self, mode: &AddressingMode) {
        let data = self.get_operand(mode);
        self.and_with_register_a(data);
        self.lsr_reg_a();
    }
    fn nop_read(&mut self, mode: &AddressingMode) {
        let data = self.get_operand(mode);
        /* do nothing */
    }
    fn rra(&mut self, mode: &AddressingMode) {
        self.ror_memory(mode);
        let data = self.get_operand(mode);
        self.add_to_reg_a(data);
    }
    fn lsb(&mut self, mode: &AddressingMode) {
        self.inc(mode);
        let data = self.get_operand(mode);
        self.sub_from_register_a(data);
    }
    fn unofficial_nop(&mut self, mode: &AddressingMode) {}
    fn lax(&mut self, mode: &AddressingMode) {
        let data = self.get_operand(mode);
        self.set_register_a(data);
        self.register.x = self.register.a;
    }
    fn sax(&mut self, mode: &AddressingMode) {
        let data = self.register.a & self.register.x;
        let addr = self.get_operand_address(mode);
        self.write(addr, data);
    }
    fn lxa(&mut self, mode: &AddressingMode) {
        self.lda(mode);
        self.tax();
    }
    fn xaa(&mut self, mode: &AddressingMode) {
        self.register.a = self.register.x;
        self.update_zero_and_negative_flags(self.register.a);
        let addr = self.get_operand_address(mode);
        let data = self.read(addr);
        self.and_with_register_a(data);
    }
    fn las(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.read(addr);
        data = data & self.register.sp;
        self.register.a = data;
        self.register.x = data;
        self.register.sp = data;
        self.update_zero_and_negative_flags(data);
    }
    fn tas(&mut self, mode: &AddressingMode) {
        let data = self.register.a & self.register.x;
        self.register.sp = data;
        let mem_address = self.read_u16(self.register.pc) + self.register.y as u16;

        let data = ((mem_address >> 8) as u8 + 1) & self.register.sp;
        self.write(mem_address, data)
    }
    fn ahx_indirect_y(&mut self, mode: &AddressingMode) {
        let pos: u8 = self.read(self.register.pc);
        let mem_address = self.read_u16(pos as u16) + self.register.y as u16;
        let data = self.register.a & self.register.x & (mem_address >> 8) as u8;
        self.write(mem_address, data)
    }
    fn ahx_absolute_y(&mut self, mode: &AddressingMode) {
        let mem_address = self.read_u16(self.register.pc) + self.register.y as u16;

        let data = self.register.a & self.register.x & (mem_address >> 8) as u8;
        self.write(mem_address, data)
    }
    fn shx(&mut self, mode: &AddressingMode) {
        let mem_address = self.read_u16(self.register.pc) + self.register.y as u16;

        // todo if cross page boundry {
        //     mem_address &= (self.x as u16) << 8;
        // }
        let data = self.register.x & ((mem_address >> 8) as u8 + 1);
        self.write(mem_address, data)
    }
    fn shy(&mut self, mode: &AddressingMode) {
        let mem_address = self.read_u16(self.register.pc) + self.register.x as u16;
        let data = self.register.y & ((mem_address >> 8) as u8 + 1);
        self.write(mem_address, data)
    }
}

impl CPU {
    pub fn new(bus: Box<dyn Addressable>) -> Self {
        CPU {
            register: Register::default(),
            bus,
        }
    }
}

/// 两字节打包成u16
fn pack_u16(high: u8, low: u8) -> u16 {
    let high = high as u16;
    let low = low as u16;
    (high << 8) | low
}
/// 解构u16(high,low)
fn unpack_u16(val: u16) -> (u8, u8) {
    let low = (val & 0xff) as u8;
    let high = (val >> 8) as u8;
    (high, low)
}

/// 读写总线的便捷方法
impl CPU {
    fn read(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }
    fn read_u16(&self, addr: u16) -> u16 {
        self.bus.read_u16(addr)
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.bus.write(addr, data);
    }
    fn write_u16(&mut self, addr: u16, data: u16) {
        self.bus.write_u16(addr, data);
    }
}

impl CPU {
    /// 获取当前的操作数
    fn get_operand(&self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        self.read(addr)
    }

    /// 获取当前的操作数的地址
    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        let pc = self.register.pc;
        let x = self.register.x;
        let y = self.register.y;
        let read = |addr: u16| self.read(addr);
        let read_u16 = |addr: u16| self.read_u16(addr);

        // 此时寄存器pc的值为指令码地址的后一个地址
        match mode {
            AddressingMode::Immediate => pc,

            AddressingMode::ZeroPage => read(pc) as u16,
            AddressingMode::Absolute => read_u16(pc),

            AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY => {
                // 零页基地址
                let base = match mode {
                    AddressingMode::ZeroPageX | AddressingMode::ZeroPageY => read(pc) as u16,
                    AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => read_u16(pc),
                    _ => panic!(),
                };
                let x_or_y = match mode {
                    AddressingMode::ZeroPageX | AddressingMode::AbsoluteX => x,
                    AddressingMode::ZeroPageY | AddressingMode::AbsoluteY => y,
                    _ => panic!(),
                };
                base.wrapping_add(x_or_y as u16)
            }
            AddressingMode::IndirectX => {
                // X间接寻址(操作码，零页基地址)
                // *(X+base) | *(X+base+1) << 8
                let base = read(pc);
                let ptr = base.wrapping_add(x);
                let low = read(ptr as u16);
                let high = read(ptr.wrapping_add(1) as u16);
                (low as u16) | ((high as u16) << 8)
            }
            AddressingMode::IndirectY => {
                // Y间接寻址(操作码，零页间接地址)
                // *(base) | *(base+1) << 8 + Y
                let base_ptr = read(pc);
                let low = read(base_ptr as u16);
                let high = read(base_ptr.wrapping_add(1) as u16);
                let base = (low as u16) | ((high as u16) << 8);
                base.wrapping_add(y as u16)
            }
            AddressingMode::NoneAddressing => panic!("mode {:?} is not supported", mode),
        }
    }
}

/// 更新标志位
impl CPU {
    /// 根据执行结果更新负数标志
    fn update_negative_flag(&mut self, result: u8) {
        // 8位整数的最高位符号位为负数标志位
        self.register.status.negative = result >> 7 == 1;
    }
    /// 根据执行结果更新零标志
    fn update_zero_flag(&mut self, result: u8) {
        // 是否为0
        self.register.status.zero = result == 0;
    }
    /// 根据执行结果更新零标志和负数标志
    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.update_zero_flag(result);
        self.update_negative_flag(result);
    }
    /// 根据结果更新进位标志
    fn update_carry_flag(&mut self, result: u16) {
        self.register.status.carry = result > 0xFF;
    }
}

impl CPU {
    fn set_register_a(&mut self, value: u8) {
        self.register.a = value;
        self.update_zero_and_negative_flags(self.register.a);
    }

    fn sub_from_register_a(&mut self, data: u8) {
        self.add_to_reg_a(((data as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }
    /// 向累加器A添加一个数
    fn add_to_reg_a(&mut self, data: u8) {
        let a = self.register.a as u16;
        let data = data as u16;
        let carry = self.register.status.carry as u16;
        let sum = a + data + carry;

        self.update_carry_flag(sum);
        let result = sum as u8;
        let data = data as u8;
        self.register.status.overflow = (data ^ result) & (result ^ self.register.a) & 0x80 != 0;
        self.set_register_a(result);
    }
}

/// 数据传送指令实现
impl CPU {
    fn load_register(&mut self, register_ref: *mut u8, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.read(addr);
        unsafe {
            *register_ref = data;
            self.update_zero_and_negative_flags(*register_ref);
        }
    }

    /// LDA--由存储器取数送入累加器 M→A
    fn lda(&mut self, mode: &AddressingMode) {
        let register_ptr = &mut self.register.a as *mut u8;
        self.load_register(register_ptr, mode);
    }
    /// LDX--由存储器取数送入寄存器X M→X
    fn ldx(&mut self, mode: &AddressingMode) {
        let register_ptr = &mut self.register.x as *mut u8;
        self.load_register(register_ptr, mode);
    }
    /// LDY--由存储器取数送入寄存器Y M→Y
    fn ldy(&mut self, mode: &AddressingMode) {
        let register_ptr = &mut self.register.y as *mut u8;
        self.load_register(register_ptr, mode);
    }
    /// STA--将累加器的内容送入存储器 A--M
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.write(addr, self.register.a);
    }
    /// STX--将寄存器X的内容送入存储器 X--M
    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.write(addr, self.register.x);
    }
    /// STY--将寄存器Y的内容送入存储器 Y--M
    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.write(addr, self.register.y);
    }

    /// 将源寄存器的值传递到目的寄存器
    fn transport_register(&mut self, src: u8, dist_ptr: *mut u8) {
        unsafe {
            *dist_ptr = src;
            self.update_zero_and_negative_flags(*dist_ptr);
        }
    }
    /// 将累加器A的内容送入变址寄存器X
    fn tax(&mut self) {
        let dist_ptr = &mut self.register.x as *mut u8;
        self.transport_register(self.register.a, dist_ptr)
    }
    /// 将变址寄存器X的内容送入累加器A
    fn txa(&mut self) {
        let dist_ptr = &mut self.register.a as *mut u8;
        self.transport_register(self.register.x, dist_ptr)
    }
    /// 将累加器A的内容送入变址寄存器Y
    fn tay(&mut self) {
        let dist_ptr = &mut self.register.y as *mut u8;
        self.transport_register(self.register.a, dist_ptr)
    }
    ///	将变址寄存器Y的内容送入累加器A
    fn tya(&mut self) {
        let dist_ptr = &mut self.register.a as *mut u8;
        self.transport_register(self.register.y, dist_ptr)
    }
    /// 将变址寄存器X的内容送入堆栈指针S
    fn txs(&mut self) {
        let dist_ptr = &mut self.register.sp as *mut u8;
        self.transport_register(self.register.x, dist_ptr)
    }
    /// 将堆栈指针S的内容送入变址寄存器X
    fn tsx(&mut self) {
        let dist_ptr = &mut self.register.x as *mut u8;
        self.transport_register(self.register.sp, dist_ptr)
    }
}

/// 算术运算指令实现
impl CPU {
    /// ADC--累加器,存储器,进位标志C相加,结果送累加器A  A+M+C→A
    fn adc(&mut self, mode: &AddressingMode) {
        let val = self.get_operand(mode);
        self.add_to_reg_a(val);
    }
    /// SBC--从累加器减去存储器和进位标志C,结果送累加器  A-M-C→A
    fn sbc(&mut self, mode: &AddressingMode) {
        let val = self.get_operand(mode);
        let data = val as i8;
        // add_to_reg_a函数内部完成了累加器与进位标志相加
        self.add_to_reg_a(data.wrapping_neg().wrapping_sub(1) as u8);
    }
    /// INC--存储器单元内容增1  M+1→M
    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.read(addr);
        let data = data.wrapping_add(1);
        self.write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    /// DEC--存储器单元内容减1  M-1→M
    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.read(addr);
        let data = data.wrapping_sub(1);
        self.write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    /// 寄存器X加1
    fn inx(&mut self) {
        self.register.x = self.register.x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register.x);
    }

    /// 寄存器X减1
    fn dex(&mut self) {
        self.register.x = self.register.x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register.x);
    }

    /// 寄存器Y加1
    fn iny(&mut self) {
        self.register.y = self.register.y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register.y);
    }

    /// 寄存器Y减1
    fn dey(&mut self) {
        self.register.x = self.register.x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register.y);
    }
}

impl CPU {
    fn and_with_register_a(&mut self, data: u8) {
        self.set_register_a(data & self.register.a);
    }

    fn xor_with_register_a(&mut self, data: u8) {
        self.set_register_a(data ^ self.register.a);
    }

    fn or_with_register_a(&mut self, data: u8) {
        self.set_register_a(data | self.register.a);
    }
}

/// 逻辑运算指令实现
impl CPU {
    /// AND--寄存器与累加器相与,结果送累加器  A∧M→A
    fn and(&mut self, mode: &AddressingMode) {
        let data = self.get_operand(mode);
        self.and_with_register_a(data);
    }
    /// ORA--寄存器与累加器相或,结果送累加器  A∨M→A
    fn ora(&mut self, mode: &AddressingMode) {
        let data = self.get_operand(mode);
        self.or_with_register_a(data);
    }
    /// EOR--寄存器与累加器相异或,结果送累加器  A≮M→A
    fn eor(&mut self, mode: &AddressingMode) {
        let data = self.get_operand(mode);
        self.xor_with_register_a(data);
    }
}

/// 置标志位指令实现
impl CPU {
    /// 清除进位标志
    fn clc(&mut self) {
        self.register.status.carry = false;
    }
    /// 置进位标志C  
    fn sec(&mut self) {
        self.register.status.carry = true;
    }
    /// 清除十进制运算标志D
    fn cld(&mut self) {
        self.register.status.decimal_mode = false;
    }
    /// 置十进制运算标志D
    fn sed(&mut self) {
        self.register.status.decimal_mode = true;
    }
    /// 清除溢出标志V
    fn clv(&mut self) {
        self.register.status.overflow = false;
    }
    /// 清除中断禁止指令I
    fn cli(&mut self) {
        self.register.status.interrupt_disable = false;
    }
    /// 置位中断禁止标志I
    fn sei(&mut self) {
        self.register.status.interrupt_disable = true;
    }
}

/// 比较指令辅助函数
impl CPU {
    /// 设A为比较指令的操作数
    /// 若执行指令CMP后,C=1表示无借位,即A>=M
    /// 若执行指令CMP后,C=0表示有借位,即A<M
    /// 若执行指令CMP后,Z=1表示A=M
    fn compare(&mut self, mode: &AddressingMode, compare_with: u8) {
        let data = self.get_operand(mode);
        let sub = compare_with as i16 - data as i16;
        self.register.status.carry = sub >= 0;
        self.update_zero_and_negative_flags(compare_with.wrapping_sub(data));
    }
}

/// 比较指令实现
impl CPU {
    fn cmp(&mut self, mode: &AddressingMode) {
        self.compare(mode, self.register.a);
    }
    fn cpx(&mut self, mode: &AddressingMode) {
        self.compare(mode, self.register.x);
    }
    fn cpy(&mut self, mode: &AddressingMode) {
        self.compare(mode, self.register.y);
    }
    /// BIT--位测试指令
    /// 这条指令的功能和AND指令有相同之处,那就是把累加器A同存储器单元相与,但和AND指令不同的是相与的结果不送入累加器A
    /// 另外该指令对标志位的影响也和AND指令不同
    /// 若 结果=0，那么Z=1
    /// 若 结果<>0,那么Z=0
    /// N=M的第7位
    /// V=M的第6位
    fn bit(&mut self, mode: &AddressingMode) {
        let data = self.get_operand(mode);
        let and_result = self.register.a & data;
        self.register.status.zero = and_result == 0;
        self.register.status.negative = (data >> 7) == 1;
        self.register.status.overflow = (data >> 6) == 1;
    }
}

/// 移位指令
impl CPU {
    fn asl_reg_a(&mut self) {
        let data = (self.register.a as u16) << 1;
        self.update_carry_flag(data);
        self.set_register_a(data as u8);
    }
    /// 算术左移指令ASL
    /// ASL移位功能是将字节内各位依次向左移1位，最高位移进标志位C中，最底位补0
    fn asl_memory(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode) as u16;
        let data = (self.read(addr) as u16) << 1;
        self.update_carry_flag(data);
        let data = data as u8;
        self.write(addr, data);
        self.update_zero_and_negative_flags(data);
    }
    /// 逻辑右移指令LSR
    /// 该指令功能是将字节内各位依次向右移1位，最低位移进标志位C，最高位补0.
    fn lsr_memory(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode) as u16;
        let data = self.read(addr);
        self.register.status.carry = data & 1 == 1;
        let data = data >> 1;
        self.write(addr, data);
        self.update_zero_and_negative_flags(data);
    }
    fn lsr_reg_a(&mut self) {
        let data = self.register.a;
        self.register.status.carry = data & 1 == 1;
        let data = data >> 1;
        self.set_register_a(data);
    }
    /// 循环左移指令ROL
    /// ROL的移位功能是将字节内容连同进位C一起依次向左移1位
    fn rol_memory(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode) as u16;
        let data = self.read(addr);
        let old_carry = self.register.status.carry;
        self.register.status.carry = data >> 7 == 1;
        let data = (data << 1) | (old_carry as u8);
        self.write(addr, data);
        self.update_negative_flag(data);
    }
    fn rol_reg_a(&mut self) {
        let data = self.register.a;
        let old_carry = self.register.status.carry;
        self.register.status.carry = data >> 7 == 1;
        let data = (data << 1) | (old_carry as u8);
        self.set_register_a(data);
    }
    /// 循环右移指令ROR
    /// ROR的移位功能是将字节内容连同进位C一起依次向右移1位
    fn ror_memory(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode) as u16;
        let data = self.read(addr);
        let old_carry = self.register.status.carry;
        self.register.status.carry = data & 1 == 1;
        let data = (data >> 1) | ((old_carry as u8) << 7);
        self.write(addr, data);
        self.update_negative_flag(data);
    }

    fn ror_reg_a(&mut self) {
        let data = self.register.a;
        let old_carry = self.register.status.carry;
        self.register.status.carry = data & 1 == 1;
        let data = (data >> 1) | ((old_carry as u8) << 7);
        self.set_register_a(data);
    }
}

/// 堆栈控制函数
/// 栈的push内存地址由高到低
/// 栈的pop内存地址由低到高
const STACK_START: u16 = 0x0100;
impl CPU {
    fn stack_pop(&mut self) -> u8 {
        self.register.sp = self.register.sp.wrapping_add(1);
        self.read(STACK_START + self.register.sp as u16)
    }
    fn stack_push(&mut self, data: u8) {
        self.write(STACK_START + self.register.sp as u16, data);
        self.register.sp = self.register.sp.wrapping_sub(1)
    }
    fn stack_pop_u16(&mut self) -> u16 {
        let low = self.stack_pop();
        let high = self.stack_pop();
        pack_u16(high, low)
    }
    fn stack_push_u16(&mut self, data: u16) {
        let (high, low) = unpack_u16(data);
        self.stack_push(high);
        self.stack_push(low);
    }
}

/// 堆栈指令
impl CPU {
    /// 累加器进栈指令 PHA
    /// PHA是隐含寻址方式的单字节指令，操作码是 48
    /// 功能是把累加器A的内容按堆栈指针S所指示的位置送入堆栈，然后堆栈指针减1
    /// 该指令不影响标志寄存器的状态
    fn pha(&mut self) {
        self.stack_push(self.register.a)
    }
    /// 累加器出栈指令 PLA
    /// PLA是隐含寻址方式的单字节指令，操作码是 68
    /// 功能是先让堆栈指针S+1，然后取加过1的S所指向的单元的内容，把它送累加器A
    /// 该指令影响标志寄存器P中的N，Z两标志位
    fn pla(&mut self) {
        let data = self.stack_pop();
        self.set_register_a(data);
    }
    /// 标志寄存器P进栈指令 PHP
    /// PHP是隐含寻址方式的单字节指令，操作码是 08
    /// 功能是把标志寄存器P的内容按堆栈指针S所指示的位置送入堆栈，然后堆栈指针减1
    /// 该指令不影响标志寄存器P的状态
    fn php(&mut self) {
        let mut status = self.register.status.clone();
        status.break_command = true;
        status.unused = true;
        self.stack_push(status.into())
    }
    /// PLP是隐含寻址方式的单字节指令，操作码是 28
    /// 功能是先让堆栈指针S+1，然后取加过1的S所指向的单元的内容，把它送标志寄存器P
    fn plp(&mut self) {
        let flags = self.stack_pop();
        self.register.status = StatusFlagRegister::from(flags);
        self.register.status.break_command = false;
        self.register.status.unused = true;
    }
}

/// 跳转指令
impl CPU {
    /// 有条件跳转指令
    fn branch(&mut self, condition: bool) {
        if !condition {
            return;
        }
        // 这里使用相对寻址，其相对地址为8位有符号整数
        let jump = self.read(self.register.pc) as i8;
        // 计算目标地址
        let jump_addr = (self.register.pc as i32 + jump as i32 + 1) as u16;
        self.register.pc = jump_addr;
    }

    /// jmp指令绝对寻址
    fn jmp_absolute(&mut self) {
        let addr = self.read_u16(self.register.pc);
        self.register.pc = addr;
    }

    /// jmp指令间接寻址
    fn jmp_indirect(&mut self) {
        // 获取间接地址(地址的地址(二级指针))
        let indirect_addr = self.read_u16(self.register.pc);
        // 得到直接地址(一级指针)
        let addr = self.read_u16(indirect_addr);
        // 跳转
        self.register.pc = addr;
    }

    /// 如果标志位Z=1则转移，否则继续
    fn beq(&mut self) {
        self.branch(self.register.status.zero)
    }
    ///如果标志位Z=0则转移，否则继续
    fn bne(&mut self) {
        self.branch(!self.register.status.zero)
    }
    /// 如果标志位C=1则转移，否则继续
    fn bcs(&mut self) {
        self.branch(self.register.status.carry)
    }
    /// 如果标志位C=0则转移，否则继续
    fn bcc(&mut self) {
        self.branch(!self.register.status.carry)
    }
    /// 如果标志位N=1则转移，否则继续
    fn bmi(&mut self) {
        self.branch(self.register.status.negative)
    }
    /// 如果标志位N=0则转移，否则继续
    fn bpl(&mut self) {
        self.branch(!self.register.status.negative)
    }
    /// 如果标志位V=1则转移，否则继续
    fn bvs(&mut self) {
        self.branch(self.register.status.overflow)
    }
    /// 如果标志位V=0则转移，否则继续
    fn bvc(&mut self) {
        self.branch(!self.register.status.overflow)
    }
    /// 转移到子程序指令JSR(绝对寻址)
    fn jsr(&mut self) {
        self.stack_push_u16(self.register.pc + 2 - 1);
        let jump_addr = self.read_u16(self.register.pc);
        self.register.pc = jump_addr;
    }
    /// 从主程序返回指令RTS(隐含寻址)
    fn rts(&mut self) {
        self.register.pc = self.stack_pop_u16() + 1;
    }
    fn rti(&mut self) {
        let status_bits = self.stack_pop();
        self.register.status = StatusFlagRegister::from(status_bits);
        self.register.status.break_command = false;
        self.register.status.unused = true;
        self.register.pc = self.stack_pop_u16();
    }
}

/// 中断指令
impl CPU {
    // fn int(&mut self) {}
}