use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
    sync::OnceLock,
};

use nes_base::{CPUState, Reader};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct NESLog {
    pub reg_pc: u16,
    pub opcode: u8,
    pub bytes: u16,
    pub instruction_abbr: String,
    pub addressing_display: String,
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub reg_status: u8,
    pub reg_sp: u8,
    pub cpu_cycles: u32,
    pub ppu_frames: u32,
    pub ppu_cycles: u32,
}

static RE: OnceLock<Regex> = OnceLock::<Regex>::new();

impl NESLog {
    /// 解析一行日志为 NesLog 结构体
    pub fn parse_line(log: &str) -> NESLog {
        let re = RE.get_or_init(|| {
            println!("Compiling regex");
            let re = Regex::new(
                r"^(\w{4})  (\w{2}) (\w{2}|  ) (\w{2}|  ) [ *]([A-Z]{3}) (.+) A:(\w{2}) X:(\w{2}) Y:(\w{2}) P:(\w{2}) SP:(\w{2}) PPU:([0-9 ]{3}),([0-9 ]{3}) CYC:(\d+)$"
            ).unwrap();
            println!("Regex compiled");
            re
        });

        let caps = re
            .captures(log)
            .ok_or_else(|| format!("parse log failed: {log}"))
            .unwrap();

        let bytes_str = format!("{}{}", &caps[4], &caps[3]).trim().to_string();

        NESLog {
            reg_pc: u16::from_str_radix(caps[1].trim(), 16).unwrap(),
            opcode: u8::from_str_radix(caps[2].trim(), 16).unwrap(),
            bytes: if !bytes_str.is_empty() {
                u16::from_str_radix(bytes_str.trim(), 16).unwrap()
            } else {
                0
            },
            instruction_abbr: caps[5].trim().to_string(),
            addressing_display: caps[6].trim().to_string(),
            reg_a: u8::from_str_radix(caps[7].trim(), 16).unwrap(),
            reg_x: u8::from_str_radix(caps[8].trim(), 16).unwrap(),
            reg_y: u8::from_str_radix(caps[9].trim(), 16).unwrap(),
            reg_status: u8::from_str_radix(caps[10].trim(), 16).unwrap(),
            reg_sp: u8::from_str_radix(caps[11].trim(), 16).unwrap(),
            ppu_frames: caps[12].trim().parse().unwrap(),
            ppu_cycles: caps[13].trim().parse().unwrap(),
            cpu_cycles: caps[14].trim().parse().unwrap(),
        }
    }
}

pub fn assert_cpu_state(bus: Rc<RefCell<dyn Reader>>, expect: &NESLog, actual: &CPUState) {
    assert_eq!(expect.reg_pc, actual.reg_pc, "PC mismatch");

    assert_eq!(
        expect.opcode,
        bus.borrow().read(actual.reg_pc),
        "Opcode mismatch"
    );

    assert_eq!(expect.reg_a, actual.reg_a, "A register mismatch");
    assert_eq!(expect.reg_x, actual.reg_x, "X register mismatch");
    assert_eq!(expect.reg_y, actual.reg_y, "Y register mismatch");
    assert_eq!(expect.reg_status, actual.reg_status, "P register mismatch");
    assert_eq!(expect.reg_sp, actual.reg_sp, "SP register mismatch");
    assert_eq!(
        expect.cpu_cycles, actual.total_cycles,
        "CPU cycles mismatch"
    );
}
