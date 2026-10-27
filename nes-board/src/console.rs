use crate::{BoardImpl, joypad::JoypadImpl};
use nes_apu::ApuImpl;
use nes_cartridge::{CartridgeImpl, NESFile};
use nes_ppu::PpuImpl;
use std::{cell::RefCell, rc::Rc};

/// Shared assembly for desktop play, deterministic replay and integration tests.
pub struct Console {
    pub board: BoardImpl,
    pub ppu: Rc<RefCell<PpuImpl>>,
    pub apu: Rc<RefCell<ApuImpl>>,
    pub controllers: [Rc<RefCell<JoypadImpl>>; 2],
}
impl Console {
    pub fn new(rom: NESFile) -> Result<Self, String> {
        let cartridge = Rc::new(RefCell::new(CartridgeImpl::try_new(rom)?));
        let ppu = Rc::new(RefCell::new(PpuImpl::new()));
        let apu = Rc::new(RefCell::new(ApuImpl::new()));
        let controllers = std::array::from_fn(|_| Rc::new(RefCell::new(JoypadImpl::default())));
        let board = BoardImpl {
            cpu_bus: Rc::new(RefCell::new(nes_bus::BusImpl::new())),
            ppu_bus: Rc::new(RefCell::new(nes_bus::BusImpl::new())),
            cpu: Rc::new(RefCell::new(nes_cpu::CpuImpl::new())),
            ppu: ppu.clone(),
            apu: apu.clone(),
            cartridge,
            ram: Rc::new(RefCell::new(nes_ram::RamImpl::new(0x800))),
            ppu_name_tables_ram: Rc::new(RefCell::new(nes_ram::RamImpl::new(0x1000))),
            ppu_palettes_tables_ram: Rc::new(RefCell::new(nes_ram::RamImpl::new(0x20))),
            joypad1: Some(controllers[0].clone()),
            joypad2: Some(controllers[1].clone()),
        }
        .init();
        Ok(Self {
            board,
            ppu,
            apu,
            controllers,
        })
    }
    pub fn run_frame(&mut self) -> Result<(), String> {
        let frame = self.ppu.borrow().frame_counter();
        while self.ppu.borrow().frame_counter() == frame {
            self.board.clock();
        }
        if let Some((pc, opcode)) = self.board.cpu.borrow().halt_reason() {
            return Err(format!(
                "CPU halted at ${pc:04X} on JAM opcode ${opcode:02X}; reset is required"
            ));
        }
        Ok(())
    }
    pub fn set_buttons(&mut self, player: usize, buttons: u8) {
        self.controllers[player].borrow_mut().set_buttons(buttons);
    }
    pub fn take_audio(&mut self) -> Vec<f32> {
        self.apu.borrow_mut().take_samples()
    }
}
