use std::{cell::RefCell, rc::Rc};

use nes_base::{APU, PPU};
use nes_board::BoardImpl;
use nes_bus::BusImpl;
use nes_cpu::CPUImpl;

mod neslog;

struct MockPPU;

impl PPU for MockPPU {
    fn write_reg_control(&mut self, value: u8) {}

    fn write_reg_mask(&mut self, value: u8) {}

    fn read_reg_status(&self) -> u8 {
        0
    }

    fn write_reg_oam_addr(&mut self, value: u8) {}

    fn read_reg_oam_data(&self) -> u8 {
        0
    }

    fn write_reg_oam_data(&mut self, value: u8) {}

    fn write_reg_scroll(&mut self, value: u8) {}

    fn write_reg_address(&mut self, value: u8) {}

    fn read_reg_data(&self) -> u8 {
        0
    }

    fn write_reg_data(&mut self, value: u8) {}

    fn reset(&mut self) {}

    fn clock(&mut self) {}

    fn attach_bus(&mut self, bus: std::rc::Rc<std::cell::RefCell<dyn nes_base::BusAdapter>>) {}
}

struct MockAPU;

impl APU for MockAPU {
    fn write_reg_pulse1_control(&mut self, value: u8) {}

    fn write_reg_pulse1_sweep(&mut self, value: u8) {}

    fn write_reg_pulse1_timer_low(&mut self, value: u8) {}

    fn write_reg_pulse1_timer_high(&mut self, value: u8) {}

    fn write_reg_pulse2_control(&mut self, value: u8) {}

    fn write_reg_pulse2_sweep(&mut self, value: u8) {}

    fn write_reg_pulse2_timer_low(&mut self, value: u8) {}

    fn write_reg_pulse2_timer_high(&mut self, value: u8) {}

    fn write_reg_triangle_control(&mut self, value: u8) {}

    fn write_reg_triangle_timer_low(&mut self, value: u8) {}

    fn write_reg_triangle_timer_high(&mut self, value: u8) {}

    fn write_reg_noise_control(&mut self, value: u8) {}

    fn write_reg_noise_period(&mut self, value: u8) {}

    fn write_reg_noise_length(&mut self, value: u8) {}

    fn write_reg_dmc_control(&mut self, value: u8) {}

    fn write_reg_dmc_value(&mut self, value: u8) {}

    fn write_reg_dmc_address(&mut self, value: u8) {}

    fn write_reg_dmc_length(&mut self, value: u8) {}

    fn write_reg_control(&mut self, value: u8) {}

    fn write_reg_frame_counter(&mut self, value: u8) {}

    fn read_reg_status(&self) -> u8 {
        0
    }

    fn clock(&mut self) {}
}

fn new_board() -> BoardImpl {
    let nes = nes_cartridge::NESFile::from_file("testfiles/nestest.nes");
    let cartridge = nes_cartridge::CartridgeImpl::new(nes);
    let board = BoardImpl {
        joypad1: None,
        joypad2: None,
        cpu_bus: Rc::new(RefCell::new(BusImpl::new())),
        ppu_bus: Rc::new(RefCell::new(BusImpl::new())),
        cpu: Rc::new(RefCell::new(CPUImpl::new())),
        ppu: Rc::new(RefCell::new(MockPPU)),
        apu: Rc::new(RefCell::new(MockAPU)),
        ram: Rc::new(RefCell::new(nes_ram::RAMImpl::new(0x800))),
        cartridge: Rc::new(RefCell::new(cartridge)),
    }
    .init();
    board
}

#[cfg(test)]
mod tests {
    use std::io::BufRead;

    use nes_cartridge::NESFile;

    use crate::neslog::{NESLog, assert_cpu_state};

    use super::*;

    #[test]
    fn it_works() {
        let mut board = new_board();
        let testlogs = std::fs::read("testfiles/nestest.txt").unwrap();

        board.reset();
        board.cpu.borrow_mut().set_reg_pc(0xc000);

        for line in testlogs.lines() {
            let log = NESLog::parse_line(&line.unwrap());
            println!("Executing: {:?}", log);
            assert_cpu_state(board.cpu_bus, &log, &board.cpu.borrow().dump_state());
            loop {
                board.cpu.borrow_mut().clock();
                if board.cpu.borrow().dump_state().remaining_cycles > 0 {
                    continue;
                }
            }
        }
    }

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
}
