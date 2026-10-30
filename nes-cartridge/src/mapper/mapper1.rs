use super::Mapper;
use nes_base::Mirroring;

pub struct Mapper1 {
    shift: u8,
    control: u8,
    chr_banks: [u8; 2],
    prg_bank: u8,
    last_write: Option<u64>,
}
impl Default for Mapper1 {
    fn default() -> Self {
        Self {
            shift: 0x10,
            control: 0x0c,
            chr_banks: [0; 2],
            prg_bank: 0,
            last_write: None,
        }
    }
}
impl Mapper for Mapper1 {
    fn prg_address(&self, addr: u16, size: usize) -> usize {
        let bank = (self.prg_bank & 0x0f) as usize;
        match (self.control >> 2) & 3 {
            0 | 1 => (bank & !1) * 0x4000 + (addr as usize & 0x7fff),
            2 => (if addr < 0xc000 { 0 } else { bank }) * 0x4000 + (addr as usize & 0x3fff),
            _ => {
                (if addr < 0xc000 {
                    bank
                } else {
                    size / 0x4000 - 1
                }) * 0x4000
                    + (addr as usize & 0x3fff)
            }
        }
    }
    fn chr_address(&self, addr: u16) -> usize {
        if self.control & 0x10 == 0 {
            (self.chr_banks[0] as usize & !1) * 0x1000 + addr as usize
        } else {
            self.chr_banks[(addr >> 12) as usize] as usize * 0x1000 + (addr as usize & 0x0fff)
        }
    }
    fn write_register(&mut self, addr: u16, value: u8, cpu_cycle: u64) {
        // 忽略 6502 读改写指令中的第二次写入。
        if self.last_write.is_some_and(|last| cpu_cycle == last + 1) {
            return;
        }
        self.last_write = Some(cpu_cycle);
        if value & 0x80 != 0 {
            self.shift = 0x10;
            self.control |= 0x0c;
            return;
        }
        let complete = self.shift & 1 != 0;
        self.shift = (self.shift >> 1) | ((value & 1) << 4);
        if complete {
            match (addr >> 13) & 3 {
                0 => self.control = self.shift,
                1 => self.chr_banks[0] = self.shift,
                2 => self.chr_banks[1] = self.shift,
                3 => self.prg_bank = self.shift,
                _ => unreachable!(),
            }
            self.shift = 0x10;
        }
    }
    fn mirroring(&self) -> Option<Mirroring> {
        Some(match self.control & 3 {
            0 => Mirroring::SingleScreen,
            1 => Mirroring::SingleScreenHigh,
            2 => Mirroring::Vertical,
            _ => Mirroring::Horizontal,
        })
    }
    fn ram_enabled(&self) -> bool {
        self.prg_bank & 0x10 == 0
    }
    fn reset(&mut self) {
        *self = Self::default();
    }
}
