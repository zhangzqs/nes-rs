use std::{cell::RefCell, rc::Rc};

use nes_base::{
    Bus, BusAdapter, CPU, Cartridge, CartridgeCPUBusAdapter, Joypad, JoypadBusAdapter, PPU,
    PPUBusAdapter, RAM, RAMBusAdapter,
};

struct BoardImpl {
    cpu_bus: Rc<RefCell<dyn Bus>>,            // CPU bus
    ppu_bus: Rc<RefCell<dyn Bus>>,            // PPU bus
    cpu: Rc<RefCell<dyn CPU>>,                // CPU
    ppu: Rc<RefCell<dyn PPU>>,                // PPU
    ram: Rc<RefCell<dyn RAM>>,                // 2KB of RAM
    cartridge: Rc<RefCell<dyn Cartridge>>,    // 游戏卡带
    joypad1: Option<Rc<RefCell<dyn Joypad>>>, // 手柄1P
    joypad2: Option<Rc<RefCell<dyn Joypad>>>, // 手柄2P
}

impl BoardImpl {
    fn attach_all(&mut self) {
        self.cpu.borrow_mut().attach_bus(self.cpu_bus.clone()); // CPU 连接到 CPU 总线上
        self.ppu.borrow_mut().attach_bus(self.ppu_bus.clone()); // PPU 连接到 PPU 总线上

        let cpu_bus_devices: [Rc<RefCell<dyn BusAdapter>>; 4] = [
            Rc::new(RefCell::new(RAMBusAdapter(self.ram.clone()))),
            Rc::new(RefCell::new(PPUBusAdapter(self.ppu.clone()))),
            Rc::new(RefCell::new(CartridgeCPUBusAdapter(self.cartridge.clone()))),
            Rc::new(RefCell::new(JoypadBusAdapter {
                joypad1: self.joypad1.clone(),
                joypad2: self.joypad2.clone(),
            })),
        ];
        for device in cpu_bus_devices {
            self.cpu_bus.borrow_mut().register_device(device);
        }
    }
}
