mod console;
pub mod joypad;
pub use console::Console;

use std::{cell::RefCell, rc::Rc};

use nes_base::{
    ApuAdapterForCpuBus, Bus, BusAdapter, Cartridge, CartridgeAdapterForCPUBus, Cpu, DmaForCpuBus,
    Interrupt, Joypad, JoypadAdapterForCpuBus, NameTablesAdapterForPpuBus,
    PalettesTablesAdapterForPpuBus, PatternTablesAdapterForPpuBus, Ppu, PpuBusAdapterForCpuBus,
    Ram, RamAdapterForCpuBus,
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
        let ppu_bus_devices: [Rc<RefCell<dyn BusAdapter>>; 3] = [
            Rc::new(RefCell::new(PatternTablesAdapterForPpuBus(
                self.cartridge.clone(),
            ))),
            Rc::new(RefCell::new(NameTablesAdapterForPpuBus {
                vram: self.ppu_name_tables_ram.clone(),
                cartridge: self.cartridge.clone(),
            })),
            Rc::new(RefCell::new(PalettesTablesAdapterForPpuBus {
                vram: self.ppu_palettes_tables_ram.clone(),
            })),
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
                apu: self.apu.clone(),
            })),
            Rc::new(RefCell::new(ApuAdapterForCpuBus(self.apu.clone()))),
            Rc::new(RefCell::new(DmaForCpuBus::default())),
        ];
        for device in cpu_bus_devices {
            self.cpu_bus.borrow_mut().register_device(device);
        }
    }

    pub fn reset(&mut self) {
        self.cartridge.borrow_mut().reset();
        self.cpu.borrow_mut().reset(); // 重置 CPU
        self.ppu.borrow_mut().reset(); // 重置 PPU
        self.apu.borrow_mut().reset();
        self.cpu_bus.borrow().take_dma_request();
    }

    pub fn clock(&mut self) {
        // Transfer only at an instruction boundary, with no CPU/bus borrow held.
        if self.cpu.borrow().remaining_cycles() == 0 {
            let page = self.cpu_bus.borrow().take_dma_request();
            if let Some(page) = page {
                for offset in 0..256 {
                    let value = self.cpu_bus.borrow().read((page as u16) << 8 | offset);
                    self.ppu.borrow_mut().write_reg_oam_data(value);
                }
                let parity = self.cpu.borrow().dump_state().total_cycles & 1;
                self.cpu.borrow_mut().increase_cycles(513 + parity);
            }
        }
        let dmc_address = self.apu.borrow_mut().take_dmc_request();
        if let Some(address) = dmc_address {
            let sample = self.cpu_bus.borrow().read(address);
            self.apu.borrow_mut().supply_dmc_sample(sample);
            self.cpu.borrow_mut().increase_cycles(4);
        }
        self.cartridge.borrow_mut().clock_cpu();
        self.cpu.borrow_mut().clock();

        let mut ppu = self.ppu.borrow_mut();
        for _ in 0..3 {
            // PPU 每个 CPU 时钟周期执行 3 次
            ppu.clock();
            if ppu.check_nmi_interrupt() {
                // 如果 PPU 检测到 NMI 中断，则触发 CPU 的 NMI 中断
                self.cpu.borrow_mut().trigger_interrupt(Interrupt::Nmi);
                ppu.clear_nmi_interrupt();
            }
        }
        drop(ppu);

        self.apu.borrow_mut().clock();
        let irq = self.apu.borrow().check_irq_interrupt() || self.cartridge.borrow().irq_pending();
        self.cpu.borrow_mut().set_irq_line(irq);
    }
}
