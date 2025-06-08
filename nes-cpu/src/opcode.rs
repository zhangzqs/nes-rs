use std::collections::HashMap;
use std::sync::LazyLock;

use crate::common::{AddressingMode, InstructionEnum};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Op {
    pub instruction: InstructionEnum,
    pub mode: AddressingMode,
    pub cycles: u8,
    pub increase_cycle_when_cross_page: bool,
}

struct OpcodeManager {
    op_table: HashMap<u8, Op>,
}

impl OpcodeManager {
    fn new() -> Self {
        use AddressingMode::*;
        use InstructionEnum::*;
        let mut op_args_table: HashMap<u8, (InstructionEnum, AddressingMode, u8, bool)> =
            HashMap::new();

        // 完整插入所有操作码
        op_args_table.insert(0x69, (ADC, Immediate, 2, false));
        op_args_table.insert(0x65, (ADC, ZeroPage, 3, false));
        op_args_table.insert(0x75, (ADC, ZeroPageX, 4, false));
        op_args_table.insert(0x6d, (ADC, Absolute, 4, false));
        op_args_table.insert(0x7d, (ADC, AbsoluteX, 4, true));
        op_args_table.insert(0x79, (ADC, AbsoluteY, 4, true));
        op_args_table.insert(0x61, (ADC, IndexedIndirect, 6, false));
        op_args_table.insert(0x71, (ADC, IndirectIndexed, 5, true));
        op_args_table.insert(0x29, (AND, Immediate, 2, false));
        op_args_table.insert(0x25, (AND, ZeroPage, 3, false));
        op_args_table.insert(0x35, (AND, ZeroPageX, 4, false));
        op_args_table.insert(0x2d, (AND, Absolute, 4, false));
        op_args_table.insert(0x3d, (AND, AbsoluteX, 4, true));
        op_args_table.insert(0x39, (AND, AbsoluteY, 4, true));
        op_args_table.insert(0x21, (AND, IndexedIndirect, 6, false));
        op_args_table.insert(0x31, (AND, IndirectIndexed, 5, true));
        op_args_table.insert(0x0a, (ASL, Accumulator, 2, false));
        op_args_table.insert(0x06, (ASL, ZeroPage, 5, false));
        op_args_table.insert(0x16, (ASL, ZeroPageX, 6, false));
        op_args_table.insert(0x0e, (ASL, Absolute, 6, false));
        op_args_table.insert(0x1e, (ASL, AbsoluteX, 7, false));
        op_args_table.insert(0x90, (BCC, Relative, 2, true));
        op_args_table.insert(0xb0, (BCS, Relative, 2, true));
        op_args_table.insert(0xf0, (BEQ, Relative, 2, true));
        op_args_table.insert(0x24, (BIT, ZeroPage, 3, false));
        op_args_table.insert(0x2c, (BIT, Absolute, 4, false));
        op_args_table.insert(0x30, (BMI, Relative, 2, true));
        op_args_table.insert(0xd0, (BNE, Relative, 2, true));
        op_args_table.insert(0x10, (BPL, Relative, 2, true));
        op_args_table.insert(0x00, (BRK, Implied, 7, false));
        op_args_table.insert(0x50, (BVC, Relative, 2, true));
        op_args_table.insert(0x70, (BVS, Relative, 2, true));
        op_args_table.insert(0x18, (CLC, Implied, 2, false));
        op_args_table.insert(0xd8, (CLD, Implied, 2, false));
        op_args_table.insert(0x58, (CLI, Implied, 2, false));
        op_args_table.insert(0xb8, (CLV, Implied, 2, false));
        op_args_table.insert(0xc9, (CMP, Immediate, 2, false));
        op_args_table.insert(0xc5, (CMP, ZeroPage, 3, false));
        op_args_table.insert(0xd5, (CMP, ZeroPageX, 4, false));
        op_args_table.insert(0xcd, (CMP, Absolute, 4, false));
        op_args_table.insert(0xdd, (CMP, AbsoluteX, 4, true));
        op_args_table.insert(0xd9, (CMP, AbsoluteY, 4, true));
        op_args_table.insert(0xc1, (CMP, IndexedIndirect, 6, true));
        op_args_table.insert(0xd1, (CMP, IndirectIndexed, 5, true));
        op_args_table.insert(0xe0, (CPX, Immediate, 2, false));
        op_args_table.insert(0xe4, (CPX, ZeroPage, 3, false));
        op_args_table.insert(0xec, (CPX, Absolute, 4, false));
        op_args_table.insert(0xc0, (CPY, Immediate, 2, false));
        op_args_table.insert(0xc4, (CPY, ZeroPage, 3, false));
        op_args_table.insert(0xcc, (CPY, Absolute, 4, false));
        op_args_table.insert(0xc6, (DEC, ZeroPage, 5, false));
        op_args_table.insert(0xd6, (DEC, ZeroPageX, 6, false));
        op_args_table.insert(0xce, (DEC, Absolute, 6, false));
        op_args_table.insert(0xde, (DEC, AbsoluteX, 7, false));
        op_args_table.insert(0xca, (DEX, Implied, 2, false));
        op_args_table.insert(0x88, (DEY, Implied, 2, false));
        op_args_table.insert(0x49, (EOR, Immediate, 2, false));
        op_args_table.insert(0x45, (EOR, ZeroPage, 3, false));
        op_args_table.insert(0x55, (EOR, ZeroPageX, 4, false));
        op_args_table.insert(0x4d, (EOR, Absolute, 4, false));
        op_args_table.insert(0x5d, (EOR, AbsoluteX, 4, true));
        op_args_table.insert(0x59, (EOR, AbsoluteY, 4, true));
        op_args_table.insert(0x41, (EOR, IndexedIndirect, 6, false));
        op_args_table.insert(0x51, (EOR, IndirectIndexed, 5, true));
        op_args_table.insert(0xe6, (INC, ZeroPage, 5, false));
        op_args_table.insert(0xf6, (INC, ZeroPageX, 6, false));
        op_args_table.insert(0xee, (INC, Absolute, 6, false));
        op_args_table.insert(0xfe, (INC, AbsoluteX, 7, false));
        op_args_table.insert(0xe8, (INX, Implied, 2, false));
        op_args_table.insert(0xc8, (INY, Implied, 2, false));
        op_args_table.insert(0x4c, (JMP, Absolute, 3, false));
        op_args_table.insert(0x6c, (JMP, Indirect, 5, false));
        op_args_table.insert(0x20, (JSR, Absolute, 6, false));
        op_args_table.insert(0xa9, (LDA, Immediate, 2, false));
        op_args_table.insert(0xa5, (LDA, ZeroPage, 3, false));
        op_args_table.insert(0xb5, (LDA, ZeroPageX, 4, false));
        op_args_table.insert(0xad, (LDA, Absolute, 4, false));
        op_args_table.insert(0xbd, (LDA, AbsoluteX, 4, true));
        op_args_table.insert(0xb9, (LDA, AbsoluteY, 4, true));
        op_args_table.insert(0xa1, (LDA, IndexedIndirect, 6, false));
        op_args_table.insert(0xb1, (LDA, IndirectIndexed, 5, true));
        op_args_table.insert(0xa2, (LDX, Immediate, 2, false));
        op_args_table.insert(0xa6, (LDX, ZeroPage, 3, false));
        op_args_table.insert(0xb6, (LDX, ZeroPageY, 4, false));
        op_args_table.insert(0xae, (LDX, Absolute, 4, false));
        op_args_table.insert(0xbe, (LDX, AbsoluteY, 4, true));
        op_args_table.insert(0xa0, (LDY, Immediate, 2, false));
        op_args_table.insert(0xa4, (LDY, ZeroPage, 3, false));
        op_args_table.insert(0xb4, (LDY, ZeroPageX, 4, false));
        op_args_table.insert(0xac, (LDY, Absolute, 4, false));
        op_args_table.insert(0xbc, (LDY, AbsoluteX, 4, true));
        op_args_table.insert(0x4a, (LSR, Accumulator, 2, false));
        op_args_table.insert(0x46, (LSR, ZeroPage, 5, false));
        op_args_table.insert(0x56, (LSR, ZeroPageX, 6, false));
        op_args_table.insert(0x4e, (LSR, Absolute, 6, false));
        op_args_table.insert(0x5e, (LSR, AbsoluteX, 7, false));
        op_args_table.insert(0x1a, (NOP, Implied, 2, false));
        op_args_table.insert(0x3a, (NOP, Implied, 2, false));
        op_args_table.insert(0x5a, (NOP, Implied, 2, false));
        op_args_table.insert(0x7a, (NOP, Implied, 2, false));
        op_args_table.insert(0xda, (NOP, Implied, 2, false));
        op_args_table.insert(0xea, (NOP, Implied, 2, false));
        op_args_table.insert(0xfa, (NOP, Implied, 2, false));
        op_args_table.insert(0x09, (ORA, Immediate, 2, false));
        op_args_table.insert(0x05, (ORA, ZeroPage, 3, false));
        op_args_table.insert(0x15, (ORA, ZeroPageX, 4, false));
        op_args_table.insert(0x0d, (ORA, Absolute, 4, false));
        op_args_table.insert(0x1d, (ORA, AbsoluteX, 4, true));
        op_args_table.insert(0x19, (ORA, AbsoluteY, 4, true));
        op_args_table.insert(0x01, (ORA, IndexedIndirect, 6, false));
        op_args_table.insert(0x11, (ORA, IndirectIndexed, 5, true));
        op_args_table.insert(0x48, (PHA, Implied, 3, false));
        op_args_table.insert(0x08, (PHP, Implied, 3, false));
        op_args_table.insert(0x68, (PLA, Implied, 4, false));
        op_args_table.insert(0x28, (PLP, Implied, 4, false));
        op_args_table.insert(0x2a, (ROL, Accumulator, 2, false));
        op_args_table.insert(0x26, (ROL, ZeroPage, 5, false));
        op_args_table.insert(0x36, (ROL, ZeroPageX, 6, false));
        op_args_table.insert(0x2e, (ROL, Absolute, 6, false));
        op_args_table.insert(0x3e, (ROL, AbsoluteX, 7, false));
        op_args_table.insert(0x6a, (ROR, Accumulator, 2, false));
        op_args_table.insert(0x66, (ROR, ZeroPage, 5, false));
        op_args_table.insert(0x76, (ROR, ZeroPageX, 6, false));
        op_args_table.insert(0x6e, (ROR, Absolute, 6, false));
        op_args_table.insert(0x7e, (ROR, AbsoluteX, 7, false));
        op_args_table.insert(0x40, (RTI, Implied, 6, false));
        op_args_table.insert(0x60, (RTS, Implied, 6, false));
        op_args_table.insert(0xeb, (SBC, Immediate, 2, false));
        op_args_table.insert(0xe9, (SBC, Immediate, 2, false));
        op_args_table.insert(0xe5, (SBC, ZeroPage, 3, false));
        op_args_table.insert(0xf5, (SBC, ZeroPageX, 4, false));
        op_args_table.insert(0xed, (SBC, Absolute, 4, false));
        op_args_table.insert(0xfd, (SBC, AbsoluteX, 4, true));
        op_args_table.insert(0xf9, (SBC, AbsoluteY, 4, true));
        op_args_table.insert(0xe1, (SBC, IndexedIndirect, 6, false));
        op_args_table.insert(0xf1, (SBC, IndirectIndexed, 5, true));
        op_args_table.insert(0x38, (SEC, Implied, 2, false));
        op_args_table.insert(0xf8, (SED, Implied, 2, false));
        op_args_table.insert(0x78, (SEI, Implied, 2, false));
        op_args_table.insert(0x85, (STA, ZeroPage, 3, false));
        op_args_table.insert(0x95, (STA, ZeroPageX, 4, false));
        op_args_table.insert(0x8d, (STA, Absolute, 4, false));
        op_args_table.insert(0x9d, (STA, AbsoluteX, 5, false));
        op_args_table.insert(0x99, (STA, AbsoluteY, 5, false));
        op_args_table.insert(0x81, (STA, IndexedIndirect, 6, false));
        op_args_table.insert(0x91, (STA, IndirectIndexed, 6, false));
        op_args_table.insert(0x86, (STX, ZeroPage, 3, false));
        op_args_table.insert(0x96, (STX, ZeroPageY, 4, false));
        op_args_table.insert(0x8e, (STX, Absolute, 4, false));
        op_args_table.insert(0x84, (STY, ZeroPage, 3, false));
        op_args_table.insert(0x94, (STY, ZeroPageX, 4, false));
        op_args_table.insert(0x8c, (STY, Absolute, 4, false));
        op_args_table.insert(0xaa, (TAX, Implied, 2, false));
        op_args_table.insert(0xa8, (TAY, Implied, 2, false));
        op_args_table.insert(0xba, (TSX, Implied, 2, false));
        op_args_table.insert(0x8a, (TXA, Implied, 2, false));
        op_args_table.insert(0x9a, (TXS, Implied, 2, false));
        op_args_table.insert(0x98, (TYA, Implied, 2, false));
        op_args_table.insert(0x4b, (ALR, Immediate, 2, false));
        op_args_table.insert(0x0b, (ANC, Immediate, 2, false));
        op_args_table.insert(0x2b, (ANC, Immediate, 2, false));
        op_args_table.insert(0x6b, (ARR, Immediate, 2, false));
        op_args_table.insert(0xcb, (AXS, Immediate, 2, false));
        op_args_table.insert(0xa7, (LAX, ZeroPage, 3, false));
        op_args_table.insert(0xb7, (LAX, ZeroPageY, 4, false));
        op_args_table.insert(0xaf, (LAX, Absolute, 4, false));
        op_args_table.insert(0xbf, (LAX, AbsoluteY, 4, true));
        op_args_table.insert(0xa3, (LAX, IndexedIndirect, 6, false));
        op_args_table.insert(0xb3, (LAX, IndirectIndexed, 5, true));
        op_args_table.insert(0x87, (SAX, ZeroPage, 3, false));
        op_args_table.insert(0x97, (SAX, ZeroPageY, 4, false));
        op_args_table.insert(0x8f, (SAX, Absolute, 4, false));
        op_args_table.insert(0x83, (SAX, IndexedIndirect, 6, true));
        op_args_table.insert(0xc7, (DCP, ZeroPage, 5, false));
        op_args_table.insert(0xd7, (DCP, ZeroPageX, 6, false));
        op_args_table.insert(0xcf, (DCP, Absolute, 6, false));
        op_args_table.insert(0xdf, (DCP, AbsoluteX, 7, false));
        op_args_table.insert(0xdb, (DCP, AbsoluteY, 7, false));
        op_args_table.insert(0xc3, (DCP, IndexedIndirect, 8, false));
        op_args_table.insert(0xd3, (DCP, IndirectIndexed, 8, false));
        op_args_table.insert(0xe7, (ISC, ZeroPage, 5, false));
        op_args_table.insert(0xf7, (ISC, ZeroPageX, 6, false));
        op_args_table.insert(0xef, (ISC, Absolute, 6, false));
        op_args_table.insert(0xff, (ISC, AbsoluteX, 7, false));
        op_args_table.insert(0xfb, (ISC, AbsoluteY, 7, false));
        op_args_table.insert(0xe3, (ISC, IndexedIndirect, 8, false));
        op_args_table.insert(0xf3, (ISC, IndirectIndexed, 8, false));
        op_args_table.insert(0x27, (RLA, ZeroPage, 5, false));
        op_args_table.insert(0x37, (RLA, ZeroPageX, 6, false));
        op_args_table.insert(0x2f, (RLA, Absolute, 6, false));
        op_args_table.insert(0x3f, (RLA, AbsoluteX, 7, false));
        op_args_table.insert(0x3b, (RLA, AbsoluteY, 7, false));
        op_args_table.insert(0x23, (RLA, IndexedIndirect, 8, false));
        op_args_table.insert(0x33, (RLA, IndirectIndexed, 8, false));
        op_args_table.insert(0x67, (RRA, ZeroPage, 5, false));
        op_args_table.insert(0x77, (RRA, ZeroPageX, 6, false));
        op_args_table.insert(0x6f, (RRA, Absolute, 6, false));
        op_args_table.insert(0x7f, (RRA, AbsoluteX, 7, false));
        op_args_table.insert(0x7b, (RRA, AbsoluteY, 7, false));
        op_args_table.insert(0x63, (RRA, IndexedIndirect, 8, false));
        op_args_table.insert(0x73, (RRA, IndirectIndexed, 8, false));
        op_args_table.insert(0x07, (SLO, ZeroPage, 5, false));
        op_args_table.insert(0x17, (SLO, ZeroPageX, 6, false));
        op_args_table.insert(0x0f, (SLO, Absolute, 6, false));
        op_args_table.insert(0x1f, (SLO, AbsoluteX, 7, false));
        op_args_table.insert(0x1b, (SLO, AbsoluteY, 7, false));
        op_args_table.insert(0x03, (SLO, IndexedIndirect, 8, false));
        op_args_table.insert(0x13, (SLO, IndirectIndexed, 8, false));
        op_args_table.insert(0x47, (SRE, ZeroPage, 5, false));
        op_args_table.insert(0x57, (SRE, ZeroPageX, 6, false));
        op_args_table.insert(0x4f, (SRE, Absolute, 6, false));
        op_args_table.insert(0x5f, (SRE, AbsoluteX, 7, false));
        op_args_table.insert(0x5b, (SRE, AbsoluteY, 7, false));
        op_args_table.insert(0x43, (SRE, IndexedIndirect, 8, false));
        op_args_table.insert(0x53, (SRE, IndirectIndexed, 8, false));
        op_args_table.insert(0x80, (SKB, Immediate, 2, false));
        op_args_table.insert(0x82, (SKB, Immediate, 2, false));
        op_args_table.insert(0x89, (SKB, Immediate, 2, false));
        op_args_table.insert(0xc2, (SKB, Immediate, 2, false));
        op_args_table.insert(0xe2, (SKB, Immediate, 2, false));
        op_args_table.insert(0x0c, (IGN, Absolute, 4, false));
        op_args_table.insert(0x1c, (IGN, AbsoluteX, 4, true));
        op_args_table.insert(0x3c, (IGN, AbsoluteX, 4, true));
        op_args_table.insert(0x5c, (IGN, AbsoluteX, 4, true));
        op_args_table.insert(0x7c, (IGN, AbsoluteX, 4, true));
        op_args_table.insert(0xdc, (IGN, AbsoluteX, 4, true));
        op_args_table.insert(0xfc, (IGN, AbsoluteX, 4, true));
        op_args_table.insert(0x04, (IGN, ZeroPage, 3, false));
        op_args_table.insert(0x44, (IGN, ZeroPage, 3, false));
        op_args_table.insert(0x64, (IGN, ZeroPage, 3, false));
        op_args_table.insert(0x14, (IGN, ZeroPageX, 4, false));
        op_args_table.insert(0x34, (IGN, ZeroPageX, 4, false));
        op_args_table.insert(0x54, (IGN, ZeroPageX, 4, false));
        op_args_table.insert(0x74, (IGN, ZeroPageX, 4, false));
        op_args_table.insert(0xd4, (IGN, ZeroPageX, 4, false));
        op_args_table.insert(0xf4, (IGN, ZeroPageX, 4, false));

        let mut op_table: HashMap<u8, Op> = HashMap::new();
        for (opcode, (instruction, mode, cycles, increase_cycle_when_cross_page)) in op_args_table {
            op_table.insert(
                opcode,
                Op {
                    instruction,
                    mode,
                    cycles,
                    increase_cycle_when_cross_page,
                },
            );
        }

        Self { op_table }
    }

    fn get_op(&self, opcode: u8) -> Op {
        self.op_table.get(&opcode).cloned().unwrap_or_else(|| {
            panic!(
                "Invalid opcode: {:#04x}. This opcode is not implemented or is illegal.",
                opcode
            )
        })
    }
}

// 全局单例的OpcodeManager
static OPCODE_MANAGER: LazyLock<OpcodeManager> = LazyLock::new(OpcodeManager::new);

pub fn get_op(opcode: u8) -> Op {
    OPCODE_MANAGER.get_op(opcode)
}
