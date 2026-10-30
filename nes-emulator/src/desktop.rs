use crate::{cli::Options, output::Output, replay::Replay};
use nes_board::Console;
use sdl2::{
    audio::AudioSpecDesired,
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::PixelFormatEnum,
};
use std::time::{Duration, Instant};

pub fn run(
    console: &mut Console,
    options: &Options,
    replay: &mut Replay,
    output: &mut Output,
) -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("nes-rs — P: pause | F2: reset | Esc: quit", 768, 720)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas
        .set_logical_size(256, 240)
        .map_err(|e| e.to_string())?;
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
        .map_err(|e| e.to_string())?;
    let audio = if options.mute {
        None
    } else {
        let device = sdl.audio().and_then(|audio| {
            audio.open_queue::<f32, _>(
                None,
                &AudioSpecDesired {
                    freq: Some(nes_apu::SAMPLE_RATE as i32),
                    channels: Some(1),
                    samples: Some(1024),
                },
            )
        });
        match device {
            Ok(device) => {
                device.resume();
                Some(device)
            }
            Err(error) => {
                eprintln!("Audio unavailable: {error}; continuing muted");
                None
            }
        }
    };
    let mut events = sdl.event_pump()?;
    let frame_duration = Duration::from_secs_f64(29780.5 / nes_apu::CPU_HZ as f64);
    let mut deadline = Instant::now();
    let mut frame = 0;
    let mut paused = false;
    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    repeat: false,
                    ..
                } => {
                    paused = !paused;
                    if let Some(audio) = &audio {
                        audio.clear();
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F2),
                    repeat: false,
                    ..
                } => {
                    console.board.reset();
                    if let Some(audio) = &audio {
                        audio.clear();
                    }
                }
                _ => {}
            }
        }
        if paused {
            std::thread::sleep(Duration::from_millis(10));
            deadline = Instant::now();
            continue;
        }
        if options.input.is_none() {
            let keyboard = events.keyboard_state();
            let bindings = [
                [
                    Scancode::Z,
                    Scancode::X,
                    Scancode::RShift,
                    Scancode::Return,
                    Scancode::Up,
                    Scancode::Down,
                    Scancode::Left,
                    Scancode::Right,
                ],
                [
                    Scancode::G,
                    Scancode::H,
                    Scancode::R,
                    Scancode::T,
                    Scancode::W,
                    Scancode::S,
                    Scancode::A,
                    Scancode::D,
                ],
            ];
            for (player, keys) in bindings.iter().enumerate() {
                let mask = keys.iter().enumerate().fold(0, |mask, (bit, &key)| {
                    mask | (u8::from(keyboard.is_scancode_pressed(key)) << bit)
                });
                console.set_buttons(player, mask);
            }
        }
        replay.apply(frame, console);
        console.run_frame()?;
        let samples = console.take_audio();
        output.audio(&samples)?;
        if let Some(audio) = &audio
            && audio.size() < nes_apu::SAMPLE_RATE / 10 * 4
        {
            audio.queue_audio(&samples)?;
        }
        texture
            .update(None, console.ppu.borrow().framebuffer().rgb(), 256 * 3)
            .map_err(|e| e.to_string())?;
        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();
        frame += 1;
        if options.frames.is_some_and(|limit| frame >= limit) {
            break;
        }
        deadline += frame_duration;
        if let Some(wait) = deadline.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
        } else if Instant::now().duration_since(deadline) > frame_duration * 4 {
            deadline = Instant::now();
        }
    }
    Ok(())
}
