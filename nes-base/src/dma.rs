use std::{cell::RefCell, rc::Rc};

use crate::{Bus, BusAdapter, Cpu, Reader, Writer};

pub struct DmaForCpuBus {
    pub cpu_bus: Rc<RefCell<dyn Bus>>,
    pub cpu: Rc<RefCell<dyn Cpu>>,
}

impl Reader for DmaForCpuBus {
    fn read(&self, addr: u16) -> u8 {
        panic!("DMA read from unsupported address: {:#04X}", addr)
    }
}

impl Writer for DmaForCpuBus {
    fn write(&mut self, _: u16, data: u8) {
        let source_page = data;
        for i in 0..256 {
            let addr = (source_page as u16) << 8 | i;
            let data = self.cpu_bus.borrow().read(addr);
            self.cpu_bus.borrow_mut().write(0x2004, data);
        }
        let total_cycles = self.cpu.borrow().dump_state().total_cycles;
        // 513 cycles for DMA transfer, plus 1 cycle if total_cycles is odd
        self.cpu
            .borrow_mut()
            .increase_cycles(513 + total_cycles % 2);
    }
}

impl BusAdapter for DmaForCpuBus {
    fn address_accept(&self, addr: u16) -> bool {
        addr == 0x4014
    }
}
