use core::panic;

trait IBusDevice {
    fn address_range(&self) -> (u16, u16);
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

struct Bus {
    devices: Vec<((u16, u16), Box<dyn IBusDevice>)>,
}

impl Bus {
    fn new() -> Self {
        Bus {
            devices: Vec::new(),
        }
    }

    fn register_device(&mut self, device: Box<dyn IBusDevice>) {
        let range = device.address_range();
        self.devices.push((range, device));
    }

    fn read(&self, address: u16) -> u8 {
        for (range, device) in &self.devices {
            if address >= range.0 && address <= range.1 {
                return device.read(address);
            }
        }
        panic!("Address out of range: {}", address);
    }

    fn write(&mut self, address: u16, value: u8) {
        for (range, device) in &mut self.devices {
            if address >= range.0 && address <= range.1 {
                device.write(address, value);
                return;
            }
        }
        panic!("Address out of range: {}", address);
    }
}

trait IRAM {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn reset(&mut self);
}

struct RAM<const SIZE: usize> {
    data: [u8; SIZE],
}

impl<const SIZE: usize> RAM<SIZE> {
    fn new() -> Self {
        RAM { data: [0; SIZE] }
    }
}

impl<const SIZE: usize> IRAM for RAM<SIZE> {
    fn read(&self, address: u16) -> u8 {
        if (address as usize) < self.data.len() {
            self.data[address as usize]
        } else {
            panic!("RAM read out of bounds: {}", address);
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if (address as usize) < self.data.len() {
            self.data[address as usize] = value;
        } else {
            panic!("RAM write out of bounds: {}", address);
        }
    }

    fn reset(&mut self) {
        for byte in &mut self.data {
            *byte = 0;
        }
    }
}

struct MainBoard {
    cpu_bus: Bus,
    ppu_bus: Bus,
    ram: RAM<0x800>,              // 2KB of RAM
    name_tables_ram: RAM<0x1000>, // 4KB for name tables
    palette_ram: RAM<0x20>,       // 32 bytes for palette
}

struct MainBoardPeripheral {

}

impl MainBoard {
    fn new() -> Self {
        MainBoard {
            cpu_bus: Bus::new(),
            ppu_bus: Bus::new(),
            ram: RAM::new(),
            name_tables_ram: RAM::new(),
            palette_ram: RAM::new(),
        }
    }
}
