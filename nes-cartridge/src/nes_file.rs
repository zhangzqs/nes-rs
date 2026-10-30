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
    /// PRG-ROM 存储块数量，每块 16 KiB。
    pub prg_banks: u8,
    /// CHR-ROM 存储块数量，每块 8 KiB。
    pub chr_banks: u8,
    /// 画面映射方式
    pub mirroring: Mirroring,
    /// 是否有电池备份，通常用于保存游戏进度
    pub has_battery_backed: bool,
    /// 是否有 Trainer ROM，通常是 512 字节
    pub has_trainer: bool,
    /// 映射器编号。
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
        let mapper_id = (bytes[7] & 0xF0) | (bytes[6] >> 4);

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
const TRAINER_SIZE: usize = 0x0200; // 512 字节

impl NESFile {
    /// 从文件路径加载 NES 文件
    pub fn from_file(path: &str) -> Self {
        Self::try_from_file(path).expect("Failed to load iNES ROM")
    }

    pub fn try_from_file(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        Self::try_new(bytes)
    }

    pub fn new(bytes: Vec<u8>) -> Self {
        Self::try_new(bytes).expect("Invalid iNES ROM")
    }

    pub fn try_new(bytes: Vec<u8>) -> Result<Self, String> {
        let raw: &[u8; 16] = bytes
            .get(..16)
            .ok_or("Truncated iNES header")?
            .try_into()
            .unwrap();
        let header = NESHeader::from(raw);
        if header.magic != *b"NES\x1a" {
            return Err("Invalid iNES signature".into());
        }
        if raw[7] & 0x0c == 0x08 {
            return Err("NES 2.0 is not supported".into());
        }
        if raw[7] & 3 != 0 {
            return Err("VS/PlayChoice ROMs are not supported".into());
        }
        if raw[9] & 1 != 0 {
            return Err("PAL ROMs are not supported; use an NTSC ROM".into());
        }
        if raw[8] > 1 {
            return Err("PRG-RAM larger than 8 KiB is not supported".into());
        }
        if header.prg_banks == 0 {
            return Err("ROM has no PRG banks".into());
        }
        let required = 16
            + if header.has_trainer { TRAINER_SIZE } else { 0 }
            + header.prg_banks as usize * PRG_BANK_SIZE
            + header.chr_banks as usize * CHR_BANK_SIZE;
        if bytes.len() < required {
            return Err(format!(
                "Truncated iNES ROM: expected {required} bytes, got {}",
                bytes.len()
            ));
        }
        Ok(Self { bytes, header })
    }

    pub fn header(&self) -> &NESHeader {
        &self.header
    }

    /// 获取 PRG-ROM 程序数据。
    pub fn prg_rom(&self) -> Vec<u8> {
        let start = self.prg_rom_start();
        let end = start + (self.header.prg_banks as usize * PRG_BANK_SIZE);
        self.bytes[start..end].to_vec()
    }

    /// 获取 CHR-ROM 图案数据。
    pub fn chr_rom(&self) -> Vec<u8> {
        let start = self.chr_rom_start();
        let end = start + (self.header.chr_banks as usize * CHR_BANK_SIZE);
        self.bytes[start..end].to_vec()
    }

    /// 获取 Trainer 数据区。
    pub fn trainer_rom(&self) -> Vec<u8> {
        if !self.header.has_trainer {
            panic!("Nes file is not support to get trainer rom");
        }
        let start = 0x10;
        let end = start + TRAINER_SIZE;
        self.bytes[start..end].to_vec()
    }

    fn prg_rom_start(&self) -> usize {
        0x10 + if self.header.has_trainer {
            TRAINER_SIZE
        } else {
            0
        }
    }

    fn chr_rom_start(&self) -> usize {
        self.prg_rom_start() + self.header.prg_banks as usize * PRG_BANK_SIZE
    }
}
