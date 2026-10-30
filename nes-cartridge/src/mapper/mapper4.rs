use super::Mapper;
use nes_base::Mirroring;

pub struct Mapper4 {
    select: u8,
    banks: [u8; 8],
    mirroring: Option<Mirroring>,
    ram_control: u8,
    irq_latch: u8,
    irq_counter: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq: bool,
    a12_high: bool,
    a12_low_since: u64,
}
impl Default for Mapper4 {
    fn default() -> Self {
        Self {
            select: 0,
            banks: [0; 8],
            mirroring: None,
            ram_control: 0x80,
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq: false,
            a12_high: false,
            a12_low_since: 0,
        }
    }
}
impl Mapper for Mapper4 {
    fn prg_address(&self, addr: u16, size: usize) -> usize {
        let second_last = size / 0x2000 - 2;
        let last = size / 0x2000 - 1;
        let banks = if self.select & 0x40 == 0 {
            [
                self.banks[6] as usize & 0x3f,
                self.banks[7] as usize & 0x3f,
                second_last,
                last,
            ]
        } else {
            [
                second_last,
                self.banks[7] as usize & 0x3f,
                self.banks[6] as usize & 0x3f,
                last,
            ]
        };
        banks[((addr - 0x8000) >> 13) as usize] * 0x2000 + (addr as usize & 0x1fff)
    }
    fn chr_address(&self, addr: u16) -> usize {
        let addr = addr ^ if self.select & 0x80 != 0 { 0x1000 } else { 0 };
        let bank = match addr >> 10 {
            0 => self.banks[0] & !1,
            1 => self.banks[0] | 1,
            2 => self.banks[1] & !1,
            3 => self.banks[1] | 1,
            slot => self.banks[(slot - 2) as usize],
        };
        bank as usize * 0x400 + (addr as usize & 0x3ff)
    }
    fn write_register(&mut self, addr: u16, value: u8, _: u64) {
        match addr & 0xe001 {
            0x8000 => self.select = value,
            0x8001 => self.banks[(self.select & 7) as usize] = value,
            0xa000 => {
                self.mirroring = Some(if value & 1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                })
            }
            0xa001 => self.ram_control = value,
            0xc000 => self.irq_latch = value,
            0xc001 => self.irq_reload = true,
            0xe000 => {
                self.irq_enabled = false;
                self.irq = false;
            }
            0xe001 => self.irq_enabled = true,
            _ => unreachable!(),
        }
    }
    fn observe_ppu_address(&mut self, addr: u16, cycle: u64) {
        let high = addr & 0x1000 != 0;
        if !high && self.a12_high {
            self.a12_low_since = cycle;
        }
        // Ignore the short low intervals between background pattern fetches.
        // Three CPU cycles (nine PPU dots) distinguish scanline edges.
        if high && !self.a12_high && cycle.saturating_sub(self.a12_low_since) >= 9 {
            if self.irq_counter == 0 || self.irq_reload {
                self.irq_counter = self.irq_latch;
            } else {
                self.irq_counter -= 1;
            }
            self.irq_reload = false;
            if self.irq_counter == 0 && self.irq_enabled {
                self.irq = true;
            }
        }
        self.a12_high = high;
    }
    fn mirroring(&self) -> Option<Mirroring> {
        self.mirroring
    }
    fn ram_enabled(&self) -> bool {
        self.ram_control & 0x80 != 0
    }
    fn ram_writable(&self) -> bool {
        self.ram_control & 0xc0 == 0x80
    }
    fn irq_pending(&self) -> bool {
        self.irq
    }
    fn reset(&mut self) {
        *self = Self::default();
    }
}
