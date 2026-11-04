use nes_board::Console;
use nes_cartridge::NESFile;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Emulator {
    console: Console,
    mapper: u8,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(rom: &[u8]) -> Result<Emulator, JsValue> {
        let rom = NESFile::try_new(rom.to_vec()).map_err(|e| JsValue::from_str(&e))?;
        let mapper = rom.header().mapper_id;
        Ok(Self {
            console: Console::new(rom).map_err(|e| JsValue::from_str(&e))?,
            mapper,
        })
    }

    pub fn mapper(&self) -> u8 {
        self.mapper
    }

    pub fn frame(&mut self) -> Result<Vec<u8>, JsValue> {
        self.console
            .run_frame()
            .map_err(|e| JsValue::from_str(&e))?;
        let ppu = self.console.ppu.borrow();
        let rgb = ppu.framebuffer().rgb();
        let mut rgba = Vec::with_capacity(256 * 240 * 4);
        for pixel in rgb.as_chunks::<3>().0 {
            rgba.extend_from_slice(pixel);
            rgba.push(255);
        }
        Ok(rgba)
    }

    pub fn audio(&mut self) -> Vec<f32> {
        self.console.take_audio()
    }

    pub fn buttons(&mut self, player: u8, mask: u8) {
        if player < 2 {
            self.console.set_buttons(player as usize, mask);
        }
    }

    pub fn reset(&mut self) {
        self.console.board.reset();
    }

    pub fn save_ram(&self) -> Vec<u8> {
        self.console
            .board
            .cartridge
            .borrow()
            .save_ram()
            .unwrap_or_default()
    }

    pub fn load_save_ram(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.console
            .board
            .cartridge
            .borrow_mut()
            .load_save_ram(data)
            .map_err(|e| JsValue::from_str(&e))
    }
}
