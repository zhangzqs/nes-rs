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
        let mut op_args_table: HashMap<u8, (InstructionEnum, AddressingMode, u8, bool)> =
            HashMap::new();

        // 完整插入所有操作码
        op_args_table.insert(
            0x69,
            (InstructionEnum::ADC, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0x65,
            (InstructionEnum::ADC, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x75,
            (InstructionEnum::ADC, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0x6d,
            (InstructionEnum::ADC, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0x7d,
            (InstructionEnum::ADC, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0x79,
            (InstructionEnum::ADC, AddressingMode::AbsoluteY, 4, true),
        );
        op_args_table.insert(
            0x61,
            (
                InstructionEnum::ADC,
                AddressingMode::IndexedIndirect,
                6,
                false,
            ),
        );
        op_args_table.insert(
            0x71,
            (
                InstructionEnum::ADC,
                AddressingMode::IndirectIndexed,
                5,
                true,
            ),
        );

        op_args_table.insert(
            0x29,
            (InstructionEnum::AND, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0x25,
            (InstructionEnum::AND, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x35,
            (InstructionEnum::AND, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0x2d,
            (InstructionEnum::AND, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0x3d,
            (InstructionEnum::AND, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0x39,
            (InstructionEnum::AND, AddressingMode::AbsoluteY, 4, true),
        );
        op_args_table.insert(
            0x21,
            (
                InstructionEnum::AND,
                AddressingMode::IndexedIndirect,
                6,
                false,
            ),
        );
        op_args_table.insert(
            0x31,
            (
                InstructionEnum::AND,
                AddressingMode::IndirectIndexed,
                5,
                true,
            ),
        );

        op_args_table.insert(
            0x0a,
            (InstructionEnum::ASL, AddressingMode::Accumulator, 2, false),
        );
        op_args_table.insert(
            0x06,
            (InstructionEnum::ASL, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0x16,
            (InstructionEnum::ASL, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0x0e,
            (InstructionEnum::ASL, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0x1e,
            (InstructionEnum::ASL, AddressingMode::AbsoluteX, 7, false),
        );

        op_args_table.insert(
            0x90,
            (InstructionEnum::BCC, AddressingMode::Relative, 2, true),
        );
        op_args_table.insert(
            0xb0,
            (InstructionEnum::BCS, AddressingMode::Relative, 2, true),
        );
        op_args_table.insert(
            0xf0,
            (InstructionEnum::BEQ, AddressingMode::Relative, 2, true),
        );

        op_args_table.insert(
            0x24,
            (InstructionEnum::BIT, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x2c,
            (InstructionEnum::BIT, AddressingMode::Absolute, 4, false),
        );

        op_args_table.insert(
            0x30,
            (InstructionEnum::BMI, AddressingMode::Relative, 2, true),
        );
        op_args_table.insert(
            0xd0,
            (InstructionEnum::BNE, AddressingMode::Relative, 2, true),
        );
        op_args_table.insert(
            0x10,
            (InstructionEnum::BPL, AddressingMode::Relative, 2, true),
        );
        op_args_table.insert(
            0x00,
            (InstructionEnum::BRK, AddressingMode::Implied, 7, false),
        );
        op_args_table.insert(
            0x50,
            (InstructionEnum::BVC, AddressingMode::Relative, 2, true),
        );
        op_args_table.insert(
            0x70,
            (InstructionEnum::BVS, AddressingMode::Relative, 2, true),
        );

        op_args_table.insert(
            0x18,
            (InstructionEnum::CLC, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0xd8,
            (InstructionEnum::CLD, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0x58,
            (InstructionEnum::CLI, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0xb8,
            (InstructionEnum::CLV, AddressingMode::Implied, 2, false),
        );

        op_args_table.insert(
            0xc9,
            (InstructionEnum::CMP, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0xc5,
            (InstructionEnum::CMP, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0xd5,
            (InstructionEnum::CMP, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0xcd,
            (InstructionEnum::CMP, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0xdd,
            (InstructionEnum::CMP, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0xd9,
            (InstructionEnum::CMP, AddressingMode::AbsoluteY, 4, true),
        );
        op_args_table.insert(
            0xc1,
            (
                InstructionEnum::CMP,
                AddressingMode::IndexedIndirect,
                6,
                true,
            ),
        );
        op_args_table.insert(
            0xd1,
            (
                InstructionEnum::CMP,
                AddressingMode::IndirectIndexed,
                5,
                true,
            ),
        );

        op_args_table.insert(
            0xe0,
            (InstructionEnum::CPX, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0xe4,
            (InstructionEnum::CPX, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0xec,
            (InstructionEnum::CPX, AddressingMode::Absolute, 4, false),
        );

        op_args_table.insert(
            0xc0,
            (InstructionEnum::CPY, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0xc4,
            (InstructionEnum::CPY, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0xcc,
            (InstructionEnum::CPY, AddressingMode::Absolute, 4, false),
        );

        op_args_table.insert(
            0xc6,
            (InstructionEnum::DEC, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0xd6,
            (InstructionEnum::DEC, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0xce,
            (InstructionEnum::DEC, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0xde,
            (InstructionEnum::DEC, AddressingMode::AbsoluteX, 7, false),
        );

        op_args_table.insert(
            0xca,
            (InstructionEnum::DEX, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0x88,
            (InstructionEnum::DEY, AddressingMode::Implied, 2, false),
        );

        op_args_table.insert(
            0x49,
            (InstructionEnum::EOR, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0x45,
            (InstructionEnum::EOR, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x55,
            (InstructionEnum::EOR, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0x4d,
            (InstructionEnum::EOR, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0x5d,
            (InstructionEnum::EOR, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0x59,
            (InstructionEnum::EOR, AddressingMode::AbsoluteY, 4, true),
        );
        op_args_table.insert(
            0x41,
            (
                InstructionEnum::EOR,
                AddressingMode::IndexedIndirect,
                6,
                false,
            ),
        );
        op_args_table.insert(
            0x51,
            (
                InstructionEnum::EOR,
                AddressingMode::IndirectIndexed,
                5,
                true,
            ),
        );

        op_args_table.insert(
            0xe6,
            (InstructionEnum::INC, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0xf6,
            (InstructionEnum::INC, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0xee,
            (InstructionEnum::INC, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0xfe,
            (InstructionEnum::INC, AddressingMode::AbsoluteX, 7, false),
        );

        op_args_table.insert(
            0xe8,
            (InstructionEnum::INX, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0xc8,
            (InstructionEnum::INY, AddressingMode::Implied, 2, false),
        );

        op_args_table.insert(
            0x4c,
            (InstructionEnum::JMP, AddressingMode::Absolute, 3, false),
        );
        op_args_table.insert(
            0x6c,
            (InstructionEnum::JMP, AddressingMode::Indirect, 5, false),
        );
        op_args_table.insert(
            0x20,
            (InstructionEnum::JSR, AddressingMode::Absolute, 6, false),
        );

        op_args_table.insert(
            0xa9,
            (InstructionEnum::LDA, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0xa5,
            (InstructionEnum::LDA, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0xb5,
            (InstructionEnum::LDA, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0xad,
            (InstructionEnum::LDA, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0xbd,
            (InstructionEnum::LDA, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0xb9,
            (InstructionEnum::LDA, AddressingMode::AbsoluteY, 4, true),
        );
        op_args_table.insert(
            0xa1,
            (
                InstructionEnum::LDA,
                AddressingMode::IndexedIndirect,
                6,
                false,
            ),
        );
        op_args_table.insert(
            0xb1,
            (
                InstructionEnum::LDA,
                AddressingMode::IndirectIndexed,
                5,
                true,
            ),
        );

        op_args_table.insert(
            0xa2,
            (InstructionEnum::LDX, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0xa6,
            (InstructionEnum::LDX, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0xb6,
            (InstructionEnum::LDX, AddressingMode::ZeroPageY, 4, false),
        );
        op_args_table.insert(
            0xae,
            (InstructionEnum::LDX, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0xbe,
            (InstructionEnum::LDX, AddressingMode::AbsoluteY, 4, true),
        );

        op_args_table.insert(
            0xa0,
            (InstructionEnum::LDY, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0xa4,
            (InstructionEnum::LDY, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0xb4,
            (InstructionEnum::LDY, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0xac,
            (InstructionEnum::LDY, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0xbc,
            (InstructionEnum::LDY, AddressingMode::AbsoluteX, 4, true),
        );

        op_args_table.insert(
            0x4a,
            (InstructionEnum::LSR, AddressingMode::Accumulator, 2, false),
        );
        op_args_table.insert(
            0x46,
            (InstructionEnum::LSR, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0x56,
            (InstructionEnum::LSR, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0x4e,
            (InstructionEnum::LSR, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0x5e,
            (InstructionEnum::LSR, AddressingMode::AbsoluteX, 7, false),
        );

        op_args_table.insert(
            0x1a,
            (InstructionEnum::NOP, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0x3a,
            (InstructionEnum::NOP, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0x5a,
            (InstructionEnum::NOP, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0x7a,
            (InstructionEnum::NOP, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0xda,
            (InstructionEnum::NOP, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0xea,
            (InstructionEnum::NOP, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0xfa,
            (InstructionEnum::NOP, AddressingMode::Implied, 2, false),
        );

        op_args_table.insert(
            0x09,
            (InstructionEnum::ORA, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0x05,
            (InstructionEnum::ORA, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x15,
            (InstructionEnum::ORA, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0x0d,
            (InstructionEnum::ORA, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0x1d,
            (InstructionEnum::ORA, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0x19,
            (InstructionEnum::ORA, AddressingMode::AbsoluteY, 4, true),
        );
        op_args_table.insert(
            0x01,
            (
                InstructionEnum::ORA,
                AddressingMode::IndexedIndirect,
                6,
                false,
            ),
        );
        op_args_table.insert(
            0x11,
            (
                InstructionEnum::ORA,
                AddressingMode::IndirectIndexed,
                5,
                true,
            ),
        );

        op_args_table.insert(
            0x48,
            (InstructionEnum::PHA, AddressingMode::Implied, 3, false),
        );
        op_args_table.insert(
            0x08,
            (InstructionEnum::PHP, AddressingMode::Implied, 3, false),
        );
        op_args_table.insert(
            0x68,
            (InstructionEnum::PLA, AddressingMode::Implied, 4, false),
        );
        op_args_table.insert(
            0x28,
            (InstructionEnum::PLP, AddressingMode::Implied, 4, false),
        );

        op_args_table.insert(
            0x2a,
            (InstructionEnum::ROL, AddressingMode::Accumulator, 2, false),
        );
        op_args_table.insert(
            0x26,
            (InstructionEnum::ROL, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0x36,
            (InstructionEnum::ROL, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0x2e,
            (InstructionEnum::ROL, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0x3e,
            (InstructionEnum::ROL, AddressingMode::AbsoluteX, 7, false),
        );

        op_args_table.insert(
            0x6a,
            (InstructionEnum::ROR, AddressingMode::Accumulator, 2, false),
        );
        op_args_table.insert(
            0x66,
            (InstructionEnum::ROR, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0x76,
            (InstructionEnum::ROR, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0x6e,
            (InstructionEnum::ROR, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0x7e,
            (InstructionEnum::ROR, AddressingMode::AbsoluteX, 7, false),
        );

        op_args_table.insert(
            0x40,
            (InstructionEnum::RTI, AddressingMode::Implied, 6, false),
        );
        op_args_table.insert(
            0x60,
            (InstructionEnum::RTS, AddressingMode::Implied, 6, false),
        );

        op_args_table.insert(
            0xeb,
            (InstructionEnum::SBC, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0xe9,
            (InstructionEnum::SBC, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0xe5,
            (InstructionEnum::SBC, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0xf5,
            (InstructionEnum::SBC, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0xed,
            (InstructionEnum::SBC, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0xfd,
            (InstructionEnum::SBC, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0xf9,
            (InstructionEnum::SBC, AddressingMode::AbsoluteY, 4, true),
        );
        op_args_table.insert(
            0xe1,
            (
                InstructionEnum::SBC,
                AddressingMode::IndexedIndirect,
                6,
                true,
            ),
        );
        op_args_table.insert(
            0xf1,
            (
                InstructionEnum::SBC,
                AddressingMode::IndirectIndexed,
                5,
                true,
            ),
        );

        op_args_table.insert(
            0x38,
            (InstructionEnum::SEC, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0xf8,
            (InstructionEnum::SED, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0x78,
            (InstructionEnum::SEI, AddressingMode::Implied, 2, false),
        );

        op_args_table.insert(
            0x85,
            (InstructionEnum::STA, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x95,
            (InstructionEnum::STA, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0x8d,
            (InstructionEnum::STA, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0x9d,
            (InstructionEnum::STA, AddressingMode::AbsoluteX, 5, false),
        );
        op_args_table.insert(
            0x99,
            (InstructionEnum::STA, AddressingMode::AbsoluteY, 5, false),
        );
        op_args_table.insert(
            0x81,
            (
                InstructionEnum::STA,
                AddressingMode::IndexedIndirect,
                6,
                false,
            ),
        );
        op_args_table.insert(
            0x91,
            (
                InstructionEnum::STA,
                AddressingMode::IndirectIndexed,
                6,
                false,
            ),
        );

        op_args_table.insert(
            0x86,
            (InstructionEnum::STX, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x96,
            (InstructionEnum::STX, AddressingMode::ZeroPageY, 4, false),
        );
        op_args_table.insert(
            0x8e,
            (InstructionEnum::STX, AddressingMode::Absolute, 4, false),
        );

        op_args_table.insert(
            0x84,
            (InstructionEnum::STY, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x94,
            (InstructionEnum::STY, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0x8c,
            (InstructionEnum::STY, AddressingMode::Absolute, 4, false),
        );

        op_args_table.insert(
            0xaa,
            (InstructionEnum::TAX, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0xa8,
            (InstructionEnum::TAY, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0xba,
            (InstructionEnum::TSX, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0x8a,
            (InstructionEnum::TXA, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0x9a,
            (InstructionEnum::TXS, AddressingMode::Implied, 2, false),
        );
        op_args_table.insert(
            0x98,
            (InstructionEnum::TYA, AddressingMode::Implied, 2, false),
        );

        // 非法指令
        op_args_table.insert(
            0x4b,
            (InstructionEnum::ALR, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0x0b,
            (InstructionEnum::ANC, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0x2b,
            (InstructionEnum::ANC, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0x6b,
            (InstructionEnum::ARR, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0xcb,
            (InstructionEnum::AXS, AddressingMode::Immediate, 2, false),
        );

        op_args_table.insert(
            0xa7,
            (InstructionEnum::LAX, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0xb7,
            (InstructionEnum::LAX, AddressingMode::ZeroPageY, 4, false),
        );
        op_args_table.insert(
            0xaf,
            (InstructionEnum::LAX, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0xbf,
            (InstructionEnum::LAX, AddressingMode::AbsoluteY, 4, true),
        );
        op_args_table.insert(
            0xa3,
            (
                InstructionEnum::LAX,
                AddressingMode::IndexedIndirect,
                6,
                false,
            ),
        );
        op_args_table.insert(
            0xb3,
            (
                InstructionEnum::LAX,
                AddressingMode::IndirectIndexed,
                5,
                true,
            ),
        );

        op_args_table.insert(
            0x87,
            (InstructionEnum::SAX, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x97,
            (InstructionEnum::SAX, AddressingMode::ZeroPageY, 4, false),
        );
        op_args_table.insert(
            0x8f,
            (InstructionEnum::SAX, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0x83,
            (
                InstructionEnum::SAX,
                AddressingMode::IndexedIndirect,
                6,
                true,
            ),
        );

        op_args_table.insert(
            0xc7,
            (InstructionEnum::DCP, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0xd7,
            (InstructionEnum::DCP, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0xcf,
            (InstructionEnum::DCP, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0xdf,
            (InstructionEnum::DCP, AddressingMode::AbsoluteX, 7, false),
        );
        op_args_table.insert(
            0xdb,
            (InstructionEnum::DCP, AddressingMode::AbsoluteY, 7, false),
        );
        op_args_table.insert(
            0xc3,
            (
                InstructionEnum::DCP,
                AddressingMode::IndexedIndirect,
                8,
                false,
            ),
        );
        op_args_table.insert(
            0xd3,
            (
                InstructionEnum::DCP,
                AddressingMode::IndirectIndexed,
                8,
                false,
            ),
        );

        op_args_table.insert(
            0xe7,
            (InstructionEnum::ISC, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0xf7,
            (InstructionEnum::ISC, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0xef,
            (InstructionEnum::ISC, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0xff,
            (InstructionEnum::ISC, AddressingMode::AbsoluteX, 7, false),
        );
        op_args_table.insert(
            0xfb,
            (InstructionEnum::ISC, AddressingMode::AbsoluteY, 7, false),
        );
        op_args_table.insert(
            0xe3,
            (
                InstructionEnum::ISC,
                AddressingMode::IndexedIndirect,
                8,
                false,
            ),
        );
        op_args_table.insert(
            0xf3,
            (
                InstructionEnum::ISC,
                AddressingMode::IndirectIndexed,
                8,
                false,
            ),
        );

        op_args_table.insert(
            0x27,
            (InstructionEnum::RLA, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0x37,
            (InstructionEnum::RLA, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0x2f,
            (InstructionEnum::RLA, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0x3f,
            (InstructionEnum::RLA, AddressingMode::AbsoluteX, 7, false),
        );
        op_args_table.insert(
            0x3b,
            (InstructionEnum::RLA, AddressingMode::AbsoluteY, 7, false),
        );
        op_args_table.insert(
            0x23,
            (
                InstructionEnum::RLA,
                AddressingMode::IndexedIndirect,
                8,
                false,
            ),
        );
        op_args_table.insert(
            0x33,
            (
                InstructionEnum::RLA,
                AddressingMode::IndirectIndexed,
                8,
                false,
            ),
        );

        op_args_table.insert(
            0x67,
            (InstructionEnum::RRA, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0x77,
            (InstructionEnum::RRA, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0x6f,
            (InstructionEnum::RRA, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0x7f,
            (InstructionEnum::RRA, AddressingMode::AbsoluteX, 7, false),
        );
        op_args_table.insert(
            0x7b,
            (InstructionEnum::RRA, AddressingMode::AbsoluteY, 7, false),
        );
        op_args_table.insert(
            0x63,
            (
                InstructionEnum::RRA,
                AddressingMode::IndexedIndirect,
                8,
                false,
            ),
        );
        op_args_table.insert(
            0x73,
            (
                InstructionEnum::RRA,
                AddressingMode::IndirectIndexed,
                8,
                false,
            ),
        );

        op_args_table.insert(
            0x07,
            (InstructionEnum::SLO, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0x17,
            (InstructionEnum::SLO, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0x0f,
            (InstructionEnum::SLO, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0x1f,
            (InstructionEnum::SLO, AddressingMode::AbsoluteX, 7, false),
        );
        op_args_table.insert(
            0x1b,
            (InstructionEnum::SLO, AddressingMode::AbsoluteY, 7, false),
        );
        op_args_table.insert(
            0x03,
            (
                InstructionEnum::SLO,
                AddressingMode::IndexedIndirect,
                8,
                false,
            ),
        );
        op_args_table.insert(
            0x13,
            (
                InstructionEnum::SLO,
                AddressingMode::IndirectIndexed,
                8,
                false,
            ),
        );

        op_args_table.insert(
            0x47,
            (InstructionEnum::SRE, AddressingMode::ZeroPage, 5, false),
        );
        op_args_table.insert(
            0x57,
            (InstructionEnum::SRE, AddressingMode::ZeroPageX, 6, false),
        );
        op_args_table.insert(
            0x4f,
            (InstructionEnum::SRE, AddressingMode::Absolute, 6, false),
        );
        op_args_table.insert(
            0x5f,
            (InstructionEnum::SRE, AddressingMode::AbsoluteX, 7, false),
        );
        op_args_table.insert(
            0x5b,
            (InstructionEnum::SRE, AddressingMode::AbsoluteY, 7, false),
        );
        op_args_table.insert(
            0x43,
            (
                InstructionEnum::SRE,
                AddressingMode::IndexedIndirect,
                8,
                false,
            ),
        );
        op_args_table.insert(
            0x53,
            (
                InstructionEnum::SRE,
                AddressingMode::IndirectIndexed,
                8,
                false,
            ),
        );

        op_args_table.insert(
            0x80,
            (InstructionEnum::SKB, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0x82,
            (InstructionEnum::SKB, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0x89,
            (InstructionEnum::SKB, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0xc2,
            (InstructionEnum::SKB, AddressingMode::Immediate, 2, false),
        );
        op_args_table.insert(
            0xe2,
            (InstructionEnum::SKB, AddressingMode::Immediate, 2, false),
        );

        op_args_table.insert(
            0x0c,
            (InstructionEnum::IGN, AddressingMode::Absolute, 4, false),
        );
        op_args_table.insert(
            0x1c,
            (InstructionEnum::IGN, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0x3c,
            (InstructionEnum::IGN, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0x5c,
            (InstructionEnum::IGN, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0x7c,
            (InstructionEnum::IGN, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0xdc,
            (InstructionEnum::IGN, AddressingMode::AbsoluteX, 4, true),
        );
        op_args_table.insert(
            0xfc,
            (InstructionEnum::IGN, AddressingMode::AbsoluteX, 4, true),
        );

        op_args_table.insert(
            0x04,
            (InstructionEnum::IGN, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x44,
            (InstructionEnum::IGN, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x64,
            (InstructionEnum::IGN, AddressingMode::ZeroPage, 3, false),
        );
        op_args_table.insert(
            0x14,
            (InstructionEnum::IGN, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0x34,
            (InstructionEnum::IGN, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0x54,
            (InstructionEnum::IGN, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0x74,
            (InstructionEnum::IGN, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0xd4,
            (InstructionEnum::IGN, AddressingMode::ZeroPageX, 4, false),
        );
        op_args_table.insert(
            0xf4,
            (InstructionEnum::IGN, AddressingMode::ZeroPageX, 4, false),
        );

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
static OPCODE_MANAGER: LazyLock<OpcodeManager> = LazyLock::new(|| OpcodeManager::new());

pub fn get_op(opcode: u8) -> Op {
    OPCODE_MANAGER.get_op(opcode)
}
