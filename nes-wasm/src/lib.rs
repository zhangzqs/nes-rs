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
        let mut rgba = vec![0; 256 * 240 * 4];
        ppu.framebuffer().write_rgba(&mut rgba);
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

    /// 读取唯一的 2 KiB CPU 工作内存，避免访问有副作用的硬件寄存器。
    pub fn ram_snapshot(&self) -> Vec<u8> {
        let ram = self.console.board.ram.borrow();
        (0..0x800).map(|address| ram.read(address)).collect()
    }

    /// 向 CPU 工作内存写入一个字节；镜像地址不接受直接写入。
    pub fn write_ram(&mut self, address: u16, value: u8) -> Result<(), JsValue> {
        if address >= 0x800 {
            return Err(JsValue::from_str("RAM address must be $0000-$07FF"));
        }
        self.console.board.ram.borrow_mut().write(address, value);
        Ok(())
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
