use std::{cell::RefCell, rc::Rc};

use nes_base::{
    ApuAdapterForCpuBus, Bus, BusAdapter, Cartridge, CartridgeAdapterForCPUBus, Cpu, DmaForCpuBus,
    Interrupt, Joypad, JoypadAdapterForCpuBus, MirrorBusAdapterForPpuBus,
    NameTablesAdapterForPpuBus, PalettesTablesAdapterForPpuBus, PatternTablesAdapterForPpuBus, Ppu,
    PpuBusAdapterForCpuBus, Ram, RamAdapterForCpuBus,
};

pub struct BoardImpl {
    pub cpu_bus: Rc<RefCell<dyn Bus>>,                 // CPU bus
    pub ppu_bus: Rc<RefCell<dyn Bus>>,                 // PPU bus
    pub cpu: Rc<RefCell<dyn Cpu>>,                     // CPU
    pub ppu: Rc<RefCell<dyn Ppu>>,                     // PPU
    pub ppu_name_tables_ram: Rc<RefCell<dyn Ram>>,     // PPU 名称表 RAM
    pub ppu_palettes_tables_ram: Rc<RefCell<dyn Ram>>, // PPU 调色板表 RAM
    pub ram: Rc<RefCell<dyn Ram>>,                     // RAM
    pub apu: Rc<RefCell<dyn nes_base::Apu>>,           // APU
    pub cartridge: Rc<RefCell<dyn Cartridge>>,         // 游戏卡带
    pub joypad1: Option<Rc<RefCell<dyn Joypad>>>,      // 手柄1P
    pub joypad2: Option<Rc<RefCell<dyn Joypad>>>,      // 手柄2P
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

        // 连接各个设备到PPU总线上
        let ppu_bus_devices: [Rc<RefCell<dyn BusAdapter>>; 4] = [
            Rc::new(RefCell::new(PatternTablesAdapterForPpuBus(
                self.cartridge.clone(),
            ))),
            Rc::new(RefCell::new(NameTablesAdapterForPpuBus {
                vram: self.ppu_name_tables_ram.clone(),
                mirroring: self.cartridge.borrow().mirroring(),
            })),
            Rc::new(RefCell::new(PalettesTablesAdapterForPpuBus {
                vram: self.ppu_palettes_tables_ram.clone(),
            })),
            Rc::new(RefCell::new(MirrorBusAdapterForPpuBus(
                self.ppu_bus.clone(),
            ))),
        ];
        for device in ppu_bus_devices {
            self.ppu_bus.borrow_mut().register_device(device);
        }

        // 连接各个设备到CPU总线上
        let cpu_bus_devices: [Rc<RefCell<dyn BusAdapter>>; 6] = [
            Rc::new(RefCell::new(RamAdapterForCpuBus(self.ram.clone()))),
            Rc::new(RefCell::new(PpuBusAdapterForCpuBus(self.ppu.clone()))),
            Rc::new(RefCell::new(CartridgeAdapterForCPUBus(
                self.cartridge.clone(),
            ))),
            Rc::new(RefCell::new(JoypadAdapterForCpuBus {
                joypad1: self.joypad1.clone(),
                joypad2: self.joypad2.clone(),
            })),
            Rc::new(RefCell::new(ApuAdapterForCpuBus(self.apu.clone()))),
            Rc::new(RefCell::new(DmaForCpuBus {
                cpu_bus: self.cpu_bus.clone(),
                cpu: self.cpu.clone(),
            })),
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

    pub fn clock(&mut self) {
        self.cpu.borrow_mut().clock();

        for _ in 0..3 {
            // PPU 每个 CPU 时钟周期执行 3 次
            self.ppu.borrow_mut().clock();
            if self.ppu.borrow().check_nmi_interrupt() {
                // 如果 PPU 检测到 NMI 中断，则触发 CPU 的 NMI 中断
                self.cpu.borrow_mut().trigger_interrupt(Interrupt::Nmi);
                self.ppu.borrow_mut().clear_nmi_interrupt();
            }
        }

        self.apu.borrow_mut().clock();
        if self.apu.borrow().check_irq_interrupt() {
            // 如果 APU 检测到 IRQ 中断，则触发 CPU 的 IRQ 中断
            self.cpu.borrow_mut().trigger_interrupt(Interrupt::Irq);
            self.apu.borrow_mut().clear_irq_interrupt();
        }
    }
}
