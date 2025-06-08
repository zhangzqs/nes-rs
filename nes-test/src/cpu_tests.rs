use log::debug;
use nes_base::CpuState;

use crate::neslog::{NESLog, assert_cpu_state};
use std::io::BufRead;

use super::*;

#[test]
fn test_cpu_simple_instructions() {
    let mut board = new_board();
    board.reset();

    let program: &[u8] = &[
        0xa9, 0x10, // LDA #$10     -> A = #$10
        0x85, 0x20, // STA $20      -> $20 = #$10
        0xa9, 0x01, // LDA #$1      -> A = #$1
        0x65, 0x20, // ADC $20      -> A = #$11
        0x85, 0x21, // STA $21      -> $21=#$11
        0xe6, 0x21, // INC $21      -> $21=#$12
        0xa4, 0x21, // LDY $21      -> Y=#$12
        0xc8, // INY          -> Y=#$13
        0x00, // BRK
    ];
    for (i, &byte) in program.iter().enumerate() {
        board.cpu_bus.borrow_mut().write(0xc000 + i as u16, byte);
    }
    board.cpu.borrow_mut().set_reg_pc(0xc000);
    let run_once = || {
        loop {
            board.cpu.borrow_mut().clock();
            if board.cpu.borrow().dump_state().remaining_cycles == 0 {
                break;
            }
        }
    };
    run_once();
    assert_eq!(board.cpu.borrow().dump_state().reg_a, 0x10);
    run_once();
    assert_eq!(board.cpu_bus.borrow().read(0x20), 0x10);
    run_once();
    assert_eq!(board.cpu.borrow().dump_state().reg_a, 0x01);
    run_once();
    assert_eq!(board.cpu.borrow().dump_state().reg_a, 0x11);
    run_once();
    assert_eq!(board.cpu_bus.borrow().read(0x21), 0x11);
    run_once();
    assert_eq!(board.cpu_bus.borrow().read(0x21), 0x12);
    run_once();
    assert_eq!(board.cpu.borrow().dump_state().reg_y, 0x12);
    run_once();
    assert_eq!(board.cpu.borrow().dump_state().reg_y, 0x13);
}

#[test]
fn test_cpu_simple_flags() {
    let mut board = new_board();
    board.reset();

    let program: &[u8] = &[
        0xa9, 0xff, // LDA #$ff
        0x85, 0x30, // STA $30      -> $30 = #$ff
        0xa9, 0x01, // LDA #$1
        0x65, 0x30, // ADC $30      -> carry, A = 0
        0xa9, 0x01, // LDA #$1
        0x65, 0x30, // ADC $30      -> carry, A = 1
        0x00, // BRK
    ];
    for (i, &byte) in program.iter().enumerate() {
        board.cpu_bus.borrow_mut().write(0xc000 + i as u16, byte);
    }
    board.cpu.borrow_mut().set_reg_pc(0xc000);
    let run_once = || {
        loop {
            board.cpu.borrow_mut().clock();
            if board.cpu.borrow().dump_state().remaining_cycles == 0 {
                break;
            }
        }
    };
    run_once();
    assert_eq!(board.cpu.borrow().dump_state().reg_a, 0xff);
    run_once();
    assert_eq!(board.cpu_bus.borrow().read(0x30), 0xff);
    run_once();
    assert_eq!(board.cpu.borrow().dump_state().reg_a, 0x01);
    assert_eq!(board.cpu.borrow().dump_state().reg_status.carry, false);
    run_once();
    assert_eq!(board.cpu.borrow().dump_state().reg_a, 0x00);
    assert_eq!(board.cpu.borrow().dump_state().reg_status.carry, true);
    run_once();
    assert_eq!(board.cpu.borrow().dump_state().reg_a, 0x01);
    assert_eq!(board.cpu.borrow().dump_state().reg_status.carry, true);
}

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn test_cpu_nestest() {
    init();
    let mut board = new_board();
    let testlogs = std::fs::read("testfiles/nestest.txt").unwrap();

    board.reset();
    board.cpu.borrow_mut().set_reg_pc(0xc000);

    for (line_no, line) in testlogs.lines().enumerate() {
        debug!("--- Line {} ---", line_no + 1);
        let log = NESLog::parse_line(&line.unwrap());
        // 分别输出log和CPU状态
        debug!("Expected: {:?}", Into::<CpuState>::into(log.clone()));
        debug!("Actual:   {:?}", board.cpu.borrow().dump_state());
        assert_cpu_state(
            board.cpu_bus.clone(),
            &log,
            &board.cpu.borrow().dump_state(),
        );
        debug!("--- Passed ---");
        loop {
            board.cpu.borrow_mut().clock();
            if board.cpu.borrow().dump_state().remaining_cycles == 0 {
                break;
            }
        }
    }
}
