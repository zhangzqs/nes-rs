use nes_base::{Cartridge, Mirroring};
mod mapper;
mod nes_file;
use mapper::Mapper;
pub use nes_file::NESFile;

pub struct CartridgeImpl {
    mapper: Box<dyn Mapper>,
    mirroring: Mirroring,
    prg: Vec<u8>,
    chr: Vec<u8>,
    chr_ram: bool,
    ram: Vec<u8>,
    battery: bool,
    cpu_cycle: u64,
}
impl CartridgeImpl {
    pub fn new(nes: NESFile) -> Self {
        Self::try_new(nes).expect("Unsupported cartridge")
    }
    pub fn try_new(nes: NESFile) -> Result<Self, String> {
        let header = nes.header();
        let mapper = mapper::create(header.mapper_id)?;
        let prg = header.prg_banks;
        let chr = header.chr_banks;
        let valid = prg.is_power_of_two()
            && (chr == 0 || chr.is_power_of_two())
            && match header.mapper_id {
                0 => prg <= 2 && chr <= 1,
                1 => prg <= 16 && chr <= 16, // 标准 MMC1 板型，配备 8 KiB PRG-RAM。
                2 => prg >= 2 && chr == 0,
                3 => prg <= 2 && (1..=4).contains(&chr),
                4 => (2..=32).contains(&prg) && chr <= 32,
                7 => (2..=16).contains(&prg) && chr == 0,
                66 => (2..=8).contains(&prg) && (1..=4).contains(&chr),
                _ => false,
            };
        if !valid {
            return Err(format!(
                "Unsupported PRG/CHR layout for mapper {}",
                header.mapper_id
            ));
        }
        let mut ram = vec![0; 0x2000];
        if header.has_trainer {
            ram[0x1000..0x1200].copy_from_slice(&nes.trainer_rom());
        }
        Ok(Self {
            mapper,
            mirroring: header.mirroring,
            prg: nes.prg_rom(),
            chr: if chr == 0 {
                vec![0; 0x2000]
            } else {
                nes.chr_rom()
            },
            chr_ram: chr == 0,
            ram,
            battery: header.has_battery_backed,
            cpu_cycle: 0,
        })
    }
}
impl Cartridge for CartridgeImpl {
    fn cpu_read(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7fff if self.mapper.ram_enabled() => self.ram[addr as usize & 0x1fff],
            0x8000..=0xffff => {
                self.prg[self.mapper.prg_address(addr, self.prg.len()) % self.prg.len()]
            }
            _ => 0,
        }
    }
    fn cpu_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x6000..=0x7fff if self.mapper.ram_writable() => {
                self.ram[addr as usize & 0x1fff] = value
            }
            0x8000..=0xffff => self.mapper.write_register(addr, value, self.cpu_cycle),
            _ => {}
        }
    }
    fn ppu_read(&self, addr: u16) -> u8 {
        self.chr[self.mapper.chr_address(addr & 0x1fff) % self.chr.len()]
    }
    fn ppu_write(&mut self, addr: u16, value: u8) {
        if self.chr_ram {
            let index = self.mapper.chr_address(addr & 0x1fff) % self.chr.len();
            self.chr[index] = value;
        }
    }
    fn mirroring(&self) -> Mirroring {
        if matches!(self.mirroring, Mirroring::FourScreen) {
            self.mirroring
        } else {
            self.mapper.mirroring().unwrap_or(self.mirroring)
        }
    }
    fn clock_cpu(&mut self) {
        self.cpu_cycle = self.cpu_cycle.wrapping_add(1);
    }
    fn observe_ppu_address(&mut self, addr: u16, cycle: u64) {
        self.mapper.observe_ppu_address(addr, cycle);
    }
    fn irq_pending(&self) -> bool {
        self.mapper.irq_pending()
    }
    fn reset(&mut self) {
        self.mapper.reset();
        self.cpu_cycle = 0;
    }
    fn save_ram(&self) -> Option<Vec<u8>> {
        self.battery.then(|| self.ram.clone())
    }
    fn load_save_ram(&mut self, bytes: &[u8]) -> Result<(), String> {
        if !self.battery || bytes.len() != self.ram.len() {
            return Err("Save must match the cartridge's 8 KiB battery RAM".into());
        }
        self.ram.copy_from_slice(bytes);
        Ok(())
    }
}
