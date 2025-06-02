use nes_base::Mirroring;

trait BitOperations {
    fn get_bit(&self, bit: u8) -> bool;
}

impl BitOperations for u8 {
    fn get_bit(&self, bit: u8) -> bool {
        (self & (1 << bit)) != 0
    }
}

pub struct NESFile {
    bytes: Vec<u8>,
}

const PRG_BANK_SIZE: u16 = 0x4000;
const CHR_BANK_SIZE: u16 = 0x2000;
const TRAINER_SIZE: u16 = 0x0200;

impl NESFile {
    pub fn new(bytes: Vec<u8>) -> Self {
        let reader = Self { bytes };
        if !reader.is_valid() {
            panic!("Nes file format error");
        }
        reader
    }

    /// 从文件路径加载 NES 文件
    pub fn from_file(path: &str) -> Self {
        let bytes = std::fs::read(path)
            .unwrap_or_else(|_| panic!("Failed to read NES file from path: {}", path));
        Self::new(bytes)
    }

    /// PRG-ROM banks 的数量，每 16KB 一块
    pub fn prg_banks(&self) -> u8 {
        self.bytes[4]
    }

    /// CHR-ROM banks 的数量，每 8KB 一块
    pub fn chr_banks(&self) -> u8 {
        self.bytes[5]
    }

    /// mirroring mode
    pub fn mirroring_mode(&self) -> Mirroring {
        let is_vertical_mirror = self.bytes[6].get_bit(0);
        let four_screen = self.bytes[6].get_bit(3);

        if four_screen {
            return Mirroring::FourScreen;
        }
        if is_vertical_mirror {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        }
    }

    /// 检查是否有基于电池的备份
    pub fn has_battery_backed(&self) -> bool {
        self.bytes[6].get_bit(1)
    }

    pub fn has_trainer(&self) -> bool {
        self.bytes[6].get_bit(2)
    }

    /// Mapper ID
    pub fn mapper_id(&self) -> u8 {
        let lower_mapper_id = self.bytes[6] & 0xf0;
        let upper_mapper_id = self.bytes[7] & 0xf0;
        upper_mapper_id | (lower_mapper_id >> 4)
    }

    /// Get PRG-ROM data
    pub fn prg_rom(&self) -> Vec<u8> {
        let start = self.prg_rom_start() as usize;
        let end = start + (self.prg_banks() as usize * PRG_BANK_SIZE as usize);
        self.bytes[start..end].to_vec()
    }

    /// Get CHR-ROM data
    pub fn chr_rom(&self) -> Vec<u8> {
        let start = self.chr_rom_start() as usize;
        let end = start + (self.chr_banks() as usize * CHR_BANK_SIZE as usize);
        self.bytes[start..end].to_vec()
    }

    /// Get trainer ROM data
    pub fn trainer_rom(&self) -> Vec<u8> {
        if !self.has_trainer() {
            panic!("Nes file is not support to get trainer rom");
        }
        let start = 0x10;
        let end = start + TRAINER_SIZE as usize;
        self.bytes[start..end].to_vec()
    }

    fn prg_rom_start(&self) -> usize {
        0x10 + if self.has_trainer() { TRAINER_SIZE } else { 0 } as usize
    }

    fn chr_rom_start(&self) -> usize {
        self.prg_rom_start() + (self.prg_banks() as usize * PRG_BANK_SIZE as usize)
    }

    /// 检查文件头是否有效
    fn is_valid(&self) -> bool {
        self.bytes[0] == 0x4e
            && self.bytes[1] == 0x45
            && self.bytes[2] == 0x53
            && self.bytes[3] == 0x1a
    }
}
