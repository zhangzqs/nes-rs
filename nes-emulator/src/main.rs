use clap::Parser;
use nes_board::Console;
use nes_cartridge::NESFile;
mod audio;
mod cli;
mod desktop;
mod headless;
mod output;
mod replay;
mod save;

fn run(options: cli::Options) -> Result<(), String> {
    let mut console = Console::new(NESFile::try_from_file(&options.rom)?)?;
    if !options.headless {
        save::load(&mut console, &options.rom)?;
    }
    let mut replay = replay::Replay::load(options.input.as_deref())?;
    let mut output = output::Output::new(&options)?;
    let result = if options.headless {
        headless::run(&mut console, &options, &mut replay, &mut output)
    } else {
        desktop::run(&mut console, &options, &mut replay, &mut output)
    };
    output.finish(&options, &console)?;
    if !options.headless {
        save::store(&console, &options.rom)?;
    }
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
