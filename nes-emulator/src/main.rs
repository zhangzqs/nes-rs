mod apu;
mod joypad;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use joypad::{Button, JoypadImpl};
use nes_base::{Joypad as JoypadTrait, PPU_FRAME_PIXELS, PPU_SCREEN_HEIGHT, PPU_SCREEN_WIDTH};
use nes_board::BoardImpl;
use nes_bus::BusImpl;
use nes_cartridge::{CartridgeImpl, NESFile};
use nes_cpu::CpuImpl;
use nes_ppu::{NES_SYS_PALETTES, PpuImpl};
use nes_ram::RamImpl;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

use crate::apu::DummyApu;

const SCALE_FACTOR: u32 = 3;
const FRAME_DURATION: Duration = Duration::from_micros(16_667);

fn main() -> Result<()> {
    env_logger::init();

    let rom_path = env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("Usage: nes-emulator <rom-path>"))?;

    let nes_file = NESFile::from_file(&rom_path);
    let cartridge = Rc::new(RefCell::new(CartridgeImpl::new(nes_file)));

    let cpu_bus = Rc::new(RefCell::new(BusImpl::new()));
    let ppu_bus = Rc::new(RefCell::new(BusImpl::new()));
    let cpu = Rc::new(RefCell::new(CpuImpl::new()));
    let ppu = Rc::new(RefCell::new(PpuImpl::new()));
    let apu = Rc::new(RefCell::new(DummyApu::new()));
    let joypad = Rc::new(RefCell::new(JoypadImpl::new()));
    let joypad_for_board: Rc<RefCell<dyn JoypadTrait>> = joypad.clone();

    let mut board = BoardImpl {
        cpu_bus: cpu_bus.clone(),
        ppu_bus: ppu_bus.clone(),
        cpu: cpu.clone(),
        ppu: ppu.clone(),
        ppu_name_tables_ram: Rc::new(RefCell::new(RamImpl::new(0x1000))),
        ppu_palettes_tables_ram: Rc::new(RefCell::new(RamImpl::new(0x20))),
        ram: Rc::new(RefCell::new(RamImpl::new(0x0800))),
        apu: apu.clone(),
        cartridge: cartridge.clone(),
        joypad1: Some(joypad_for_board),
        joypad2: None,
    }
    .init();

    let sdl_context = sdl_result(sdl2::init(), "Failed to init SDL2")?;
    let video_subsystem = sdl_result(sdl_context.video(), "Failed to init SDL2 video")?;
    let window = video_subsystem
        .window(
            "nes-rs",
            PPU_SCREEN_WIDTH as u32 * SCALE_FACTOR,
            PPU_SCREEN_HEIGHT as u32 * SCALE_FACTOR,
        )
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| anyhow!("Failed to create window: {e}"))?;
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| anyhow!("Failed to create canvas: {e}"))?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGB24,
            PPU_SCREEN_WIDTH as u32,
            PPU_SCREEN_HEIGHT as u32,
        )
        .map_err(|e| anyhow!("Failed to create texture: {e}"))?;
    let mut event_pump = sdl_result(sdl_context.event_pump(), "Failed to get event pump")?;

    let mut frame_buffer = vec![0u8; PPU_FRAME_PIXELS];
    let mut running = true;
    let mut last_present = Instant::now();

    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    repeat: false,
                    ..
                } => running = false,
                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    if let Some(button) = map_key(key) {
                        joypad.borrow_mut().set_button_state(button, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    if let Some(button) = map_key(key) {
                        joypad.borrow_mut().set_button_state(button, false);
                    }
                }
                _ => {}
            }
        }

        board.clock();

        if board.ppu.borrow().frame_ready() {
            let copied = {
                let ppu_ref = board.ppu.borrow();
                ppu_ref.copy_frame(&mut frame_buffer)
            };

            if copied {
                board.ppu.borrow_mut().acknowledge_frame();
                upload_frame_to_texture(&mut texture, &frame_buffer)?;
                canvas.clear();
                let target_rect = Rect::new(
                    0,
                    0,
                    PPU_SCREEN_WIDTH as u32 * SCALE_FACTOR,
                    PPU_SCREEN_HEIGHT as u32 * SCALE_FACTOR,
                );
                sdl_result(
                    canvas.copy(&texture, None, Some(target_rect)),
                    "Failed to copy texture",
                )?;
                canvas.present();

                let elapsed = last_present.elapsed();
                if elapsed < FRAME_DURATION {
                    thread::sleep(FRAME_DURATION - elapsed);
                }
                last_present = Instant::now();
            }
        }
    }

    Ok(())
}

fn map_key(key: Keycode) -> Option<Button> {
    match key {
        Keycode::Z => Some(Button::A),
        Keycode::X => Some(Button::B),
        Keycode::RShift | Keycode::LShift => Some(Button::Select),
        Keycode::Return => Some(Button::Start),
        Keycode::Up => Some(Button::Up),
        Keycode::Down => Some(Button::Down),
        Keycode::Left => Some(Button::Left),
        Keycode::Right => Some(Button::Right),
        _ => None,
    }
}

fn upload_frame_to_texture(texture: &mut sdl2::render::Texture, pixels: &[u8]) -> Result<()> {
    texture
        .with_lock(None, |buffer, pitch| {
            for y in 0..PPU_SCREEN_HEIGHT {
                for x in 0..PPU_SCREEN_WIDTH {
                    let idx = pixels[y * PPU_SCREEN_WIDTH + x] as usize % NES_SYS_PALETTES.len();
                    let color = NES_SYS_PALETTES[idx];
                    let offset = y * pitch + x * 3;
                    buffer[offset] = (color & 0xFF) as u8;
                    buffer[offset + 1] = ((color >> 8) & 0xFF) as u8;
                    buffer[offset + 2] = ((color >> 16) & 0xFF) as u8;
                }
            }
        })
        .map_err(|e| anyhow!("Failed to upload frame to texture: {e}"))
}

fn sdl_result<T>(res: Result<T, String>, msg: &str) -> Result<T> {
    res.map_err(|e| anyhow!("{msg}: {e}"))
}
