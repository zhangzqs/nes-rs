use clap::Parser;
use nes_board::Console;
use nes_cartridge::NESFile;
mod cli;
mod desktop;
mod save;

fn run(options: cli::Options) -> Result<(), String> {
    let mut console = Console::new(NESFile::try_from_file(&options.rom)?)?;
    save::load(&mut console, &options.rom)?;
    let result = desktop::run(&mut console, options.mute);
    save::store(&console, &options.rom)?;
    result
}
fn main() -> std::process::ExitCode {
    match run(cli::Options::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nes-emulator: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
