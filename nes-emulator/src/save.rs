use nes_board::Console;
use std::{fs, path::Path};

pub fn load(console: &mut Console, rom: &Path) -> Result<(), String> {
    if console.board.cartridge.borrow().save_ram().is_none() {
        return Ok(());
    }
    let path = rom.with_extension("sav");
    match fs::read(&path) {
        Ok(bytes) => console
            .board
            .cartridge
            .borrow_mut()
            .load_save_ram(&bytes)
            .map_err(|error| format!("{}: {error}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}
pub fn store(console: &Console, rom: &Path) -> Result<(), String> {
    let Some(bytes) = console.board.cartridge.borrow().save_ram() else {
        return Ok(());
    };
    let path = rom.with_extension("sav");
    let temporary = rom.with_extension("sav.tmp");
    fs::write(&temporary, bytes)
        .and_then(|_| fs::rename(&temporary, &path))
        .map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nes_cartridge::NESFile;
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Directory(PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn console() -> Console {
        let mut rom = vec![0; 16 + 0x8000];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4] = 2;
        rom[6] = 0x12; // MMC1，带电池存档
        let program = [
            0xad, 0, 0x60, 0x18, 0x69, 1, 0x8d, 0, 0x60, 0x85, 0, 0x4c, 0x0b, 0x80,
        ];
        rom[16..16 + program.len()].copy_from_slice(&program);
        rom[16 + 0x7ffc..16 + 0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
        Console::new(NESFile::new(rom)).unwrap()
    }

    #[test]
    fn battery_save_round_trips_and_rejects_corrupt_data() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = Directory(
            std::env::temp_dir().join(format!("nes-save-{}-{unique}", std::process::id())),
        );
        fs::create_dir(&directory.0).unwrap();
        let rom = directory.0.join("存档 游戏.nes");
        let save = rom.with_extension("sav");
        for expected in [1, 2] {
            let mut nes = console();
            load(&mut nes, &rom).unwrap();
            nes.run_frame().unwrap();
            store(&nes, &rom).unwrap();
            let bytes = fs::read(&save).unwrap();
            assert_eq!(bytes.len(), 8192);
            assert_eq!(bytes[0], expected);
            assert!(!rom.with_extension("sav.tmp").exists());
        }
        fs::write(&save, [7; 3]).unwrap();
        assert!(load(&mut console(), &rom).is_err());
        assert_eq!(fs::read(save).unwrap(), [7; 3]);
    }
}
