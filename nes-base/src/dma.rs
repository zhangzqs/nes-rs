use std::{cell::RefCell, rc::Rc};

use crate::{BusAdapter, Cpu, Reader, Writer};

pub trait Dma {
    fn transfer(&mut self, source_addr: u16, dest_addr: u16, length: usize);

    /// Transfers a full page (256 bytes) from source to destination.
    fn transfer_page(&mut self, source_page: u8, dest_page: u8) {
        let source_addr = (source_page as u16) << 8;
        let dest_addr = (dest_page as u16) << 8;
        self.transfer(source_addr, dest_addr, 256);
    }
}

pub struct DmaAdapterForCpuBus {
    dma: Rc<RefCell<dyn Dma>>,
    cpu: Rc<RefCell<dyn Cpu>>,
}

impl Reader for DmaAdapterForCpuBus {
    fn read(&self, addr: u16) -> u8 {
        panic!("DMA read from unsupported address: {:#04X}", addr)
    }
}

impl Writer for DmaAdapterForCpuBus {
    fn write(&mut self, _: u16, data: u8) {
        let source_page = data;
        self.dma.borrow_mut().transfer_page(source_page, 0);
        let total_cycles = self.cpu.borrow().dump_state().total_cycles;
        // 513 cycles for DMA transfer, plus 1 cycle if total_cycles is odd
        self.cpu
            .borrow_mut()
            .increase_cycles(513 + total_cycles % 2);
    }
}

impl BusAdapter for DmaAdapterForCpuBus {
    fn address_accept(&self, addr: u16) -> bool {
        addr == 0x4014
    }
}
