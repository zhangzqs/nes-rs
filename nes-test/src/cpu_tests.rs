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
        0xa9, 0x10, // LDA：将 $10 载入 A
        0x85, 0x20, // STA：将 A 存入 $20，结果为 $10
        0xa9, 0x01, // LDA：将 $01 载入 A
        0x65, 0x20, // ADC：累加 $20 的值，A 变为 $11
        0x85, 0x21, // STA：将 A 存入 $21，结果为 $11
        0xe6, 0x21, // INC：递增 $21 的值，结果为 $12
        0xa4, 0x21, // LDY：将 $21 的值载入 Y，结果为 $12
        0xc8, // INY：递增 Y，结果为 $13
        0x00, // BRK：软件中断
    ];
    for (i, &byte) in program.iter().enumerate() {
        board.cpu_bus.borrow_mut().write(0x0600 + i as u16, byte);
    }
    board.cpu.borrow_mut().set_reg_pc(0x0600);
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
        0xa9, 0xff, // LDA：将 $FF 载入 A
        0x85, 0x30, // STA：将 A 存入 $30，结果为 $FF
        0xa9, 0x01, // LDA：将 $01 载入 A
        0x65, 0x30, // ADC：累加后进位置 1，A 为 0
        0xa9, 0x01, // LDA：将 $01 载入 A
        0x65, 0x30, // ADC：带进位累加后进位仍为 1，A 为 1
        0x00, // BRK：软件中断
    ];
    for (i, &byte) in program.iter().enumerate() {
        board.cpu_bus.borrow_mut().write(0x0600 + i as u16, byte);
    }
    board.cpu.borrow_mut().set_reg_pc(0x0600);
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
    assert!(!board.cpu.borrow().dump_state().reg_status.carry);
    run_once();
    assert_eq!(board.cpu.borrow().dump_state().reg_a, 0x00);
    assert!(board.cpu.borrow().dump_state().reg_status.carry);
    run_once();
    assert_eq!(board.cpu.borrow().dump_state().reg_a, 0x01);
    assert!(board.cpu.borrow().dump_state().reg_status.carry);
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
    for _ in 0..7 {
        board.cpu.borrow_mut().clock();
    }
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
