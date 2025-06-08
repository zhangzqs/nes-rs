use std::{cell::RefCell, rc::Rc};

use nes_base::{
    APU, APUBusAdapter, Bus, BusAdapter, CPU, Cartridge, CartridgeCPUBusAdapter, Joypad,
    JoypadBusAdapter, PPU, PPUBusAdapterForCPUBus, RAM, RAMBusAdapter,
};

pub struct BoardImpl {
    pub cpu_bus: Rc<RefCell<dyn Bus>>,            // CPU bus
    pub ppu_bus: Rc<RefCell<dyn Bus>>,            // PPU bus
    pub cpu: Rc<RefCell<dyn CPU>>,                // CPU
    pub ppu: Rc<RefCell<dyn PPU>>,                // PPU
    pub ram: Rc<RefCell<dyn RAM>>,                // RAM
    pub apu: Rc<RefCell<dyn nes_base::APU>>,      // APU
    pub cartridge: Rc<RefCell<dyn Cartridge>>,    // 游戏卡带
    pub joypad1: Option<Rc<RefCell<dyn Joypad>>>, // 手柄1P
    pub joypad2: Option<Rc<RefCell<dyn Joypad>>>, // 手柄2P
}

impl BoardImpl {
    pub fn init(mut self) -> Self {
        self.attach_all(); // 连接所有设备
        self.reset(); // 重置设备

        self
    }

    fn attach_all(&mut self) {
        self.cpu.borrow_mut().attach_bus(self.cpu_bus.clone()); // CPU 连接到 CPU 总线上
        self.ppu.borrow_mut().attach_bus(self.ppu_bus.clone()); // PPU 连接到 PPU 总线上

        let cpu_bus_devices: [Rc<RefCell<dyn BusAdapter>>; 5] = [
            Rc::new(RefCell::new(RAMBusAdapter(self.ram.clone()))),
            Rc::new(RefCell::new(PPUBusAdapterForCPUBus(self.ppu.clone()))),
            Rc::new(RefCell::new(CartridgeCPUBusAdapter(self.cartridge.clone()))),
            Rc::new(RefCell::new(JoypadBusAdapter {
                joypad1: self.joypad1.clone(),
                joypad2: self.joypad2.clone(),
            })),
            Rc::new(RefCell::new(APUBusAdapter(self.apu.clone()))),
        ];
        for device in cpu_bus_devices {
            self.cpu_bus.borrow_mut().register_device(device);
        }
    }

    pub fn reset(&mut self) {
        self.cpu.borrow_mut().reset(); // 重置 CPU
        self.ppu.borrow_mut().reset(); // 重置 PPU
        self.ram.borrow_mut().reset(); // 重置 RAM
    }
}
