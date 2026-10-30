use crate::audio::Playback;
use nes_board::Console;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::PixelFormatEnum,
};
use std::time::{Duration, Instant};

pub fn run(console: &mut Console, mute: bool) -> Result<(), String> {
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
    let audio = if mute {
        None
    } else {
        match Playback::new(&sdl) {
            Ok(device) => Some(device),
            Err(error) => {
                eprintln!("Audio unavailable: {error}; continuing muted");
                None
            }
        }
    };
    let mut events = sdl.event_pump()?;
    let frame_duration = Duration::from_secs_f64(29780.5 / nes_apu::CPU_HZ as f64);
    let mut deadline = Instant::now();
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
        if let Some(audio) = &audio {
            let wait = audio.wait_duration();
            if !wait.is_zero() {
                // 积压时暂停生成新帧，每帧检查一次窗口事件，避免设备停滞阻塞退出。
                std::thread::sleep(wait.min(frame_duration));
                deadline = Instant::now();
                continue;
            }
        }
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
        console.run_frame()?;
        let samples = console.take_audio();
        if let Some(audio) = &audio {
            audio.queue(&samples)?;
        }
        texture
            .update(None, console.ppu.borrow().framebuffer().rgb(), 256 * 3)
            .map_err(|e| e.to_string())?;
        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();
        deadline += frame_duration;
        let now = Instant::now();
        let frame_wait = deadline.saturating_duration_since(now);
        if now.saturating_duration_since(deadline) > frame_duration * 4 {
            deadline = now;
        }
        std::thread::sleep(frame_wait);
    }
    Ok(())
}
