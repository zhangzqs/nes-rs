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
