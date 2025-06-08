use crate::{
    common::{AddressingMode, is_page_crossed},
    state::Context,
};

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
