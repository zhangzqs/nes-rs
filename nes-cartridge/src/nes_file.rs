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
    header: NESHeader,
}

pub struct NESHeader {
    /// 文件头标识，必须是 "NES\x1A"
    pub magic: [u8; 4],
    /// PRG-ROM banks 的数量，每 16KB 一块
    pub prg_banks: u8,
    /// CHR-ROM banks 的数量，每 8KB 一块
    pub chr_banks: u8,
    /// 画面映射方式
    pub mirroring: Mirroring,
    /// 是否有电池备份，通常用于保存游戏进度
    pub has_battery_backed: bool,
    /// 是否有 Trainer ROM，通常是 512 字节
    pub has_trainer: bool,
    /// Mapper ID
    pub mapper_id: u8,
}

impl From<&[u8; 16]> for NESHeader {
    fn from(bytes: &[u8; 16]) -> Self {
        let magic = [bytes[0], bytes[1], bytes[2], bytes[3]];
        let prg_banks = bytes[4];
        let chr_banks = bytes[5];
        let mirroring = if bytes[6].get_bit(3) {
            Mirroring::FourScreen
        } else if bytes[6].get_bit(0) {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
        let has_battery_backed = bytes[6].get_bit(1);
        let has_trainer = bytes[6].get_bit(2);
        let mapper_id = (bytes[7] >> 4) | (bytes[6] & 0xF0);

        Self {
            magic,
            prg_banks,
            chr_banks,
            mirroring,
            has_battery_backed,
            has_trainer,
            mapper_id,
        }
    }
}

const PRG_BANK_SIZE: usize = 0x4000; // 16KB
const CHR_BANK_SIZE: usize = 0x2000; // 8KB
const TRAINER_SIZE: usize = 0x0200; // 512 bytes

impl NESFile {
    /// 从文件路径加载 NES 文件
    pub fn from_file(path: &str) -> Self {
        let bytes = std::fs::read(path)
            .unwrap_or_else(|_| panic!("Failed to read NES file from path: {}", path));
        Self::new(bytes)
    }

    pub fn new(bytes: Vec<u8>) -> Self {
        let header = NESHeader::from(&bytes[0..16].try_into().expect("Invalid NES header length"));
        if header.magic != [0x4E, 0x45, 0x53, 0x1A] {
            panic!("Invalid NES file magic number");
        }
        if header.prg_banks == 0 {
            panic!("NES file must have at least one PRG-ROM bank");
        }
        if header.chr_banks == 0 {
            panic!("NES file must have at least one CHR-ROM bank");
        }
        Self {
            bytes: bytes,
            header: header,
        }
    }

    pub fn header(&self) -> &NESHeader {
        &self.header
    }

    /// Get PRG-ROM data
    pub fn prg_rom(&self) -> Vec<u8> {
        let start = self.prg_rom_start() as usize;
        let end = start + (self.header.prg_banks as usize * PRG_BANK_SIZE as usize);
        self.bytes[start..end].to_vec()
    }

    /// Get CHR-ROM data
    pub fn chr_rom(&self) -> Vec<u8> {
        let start = self.chr_rom_start() as usize;
        let end = start + (self.header.chr_banks as usize * CHR_BANK_SIZE as usize);
        self.bytes[start..end].to_vec()
    }

    /// Get trainer ROM data
    pub fn trainer_rom(&self) -> Vec<u8> {
        if !self.header.has_trainer {
            panic!("Nes file is not support to get trainer rom");
        }
        let start = 0x10;
        let end = start + TRAINER_SIZE as usize;
        self.bytes[start..end].to_vec()
    }

    fn prg_rom_start(&self) -> usize {
        0x10 + if self.header.has_trainer {
            TRAINER_SIZE
        } else {
            0
        } as usize
    }

    fn chr_rom_start(&self) -> usize {
        self.prg_rom_start() + self.header.prg_banks as usize * PRG_BANK_SIZE as usize
    }
}
