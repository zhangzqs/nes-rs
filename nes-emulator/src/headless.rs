use crate::{cli::Options, output::Output, replay::Replay};
use nes_board::Console;

pub fn run(
    console: &mut Console,
    options: &Options,
    replay: &mut Replay,
    output: &mut Output,
) -> Result<(), String> {
    let mut requested_reset = None;
    for frame in 0..options.frames.unwrap() {
        replay.apply(frame, console);
        console.run_frame()?;
        output.audio(&console.take_audio())?;
        if options.test_rom {
            let bus = console.board.cpu_bus.borrow();
            let signature: Vec<_> = (0x6001..=0x6003).map(|addr| bus.read(addr)).collect();
            if signature == [0xde, 0xb0, 0x61] {
                let status = bus.read(0x6000);
                if status < 0x80 {
                    let message: Vec<_> = (0x6004..0x8000)
                        .map(|addr| bus.read(addr))
                        .take_while(|b| *b != 0)
                        .collect();
                    println!("{}", String::from_utf8_lossy(&message));
                    return if status == 0 {
                        Ok(())
                    } else {
                        Err(format!("Test ROM failed with status {status}"))
                    };
                }
                if status == 0x81 && requested_reset.is_none() {
                    requested_reset = Some(frame + 6);
                }
            }
        }
        if requested_reset == Some(frame) {
            console.board.reset();
            requested_reset = None;
        }
    }
    if options.test_rom {
        Err("Test ROM timed out without a result".into())
    } else {
        Ok(())
    }
}
