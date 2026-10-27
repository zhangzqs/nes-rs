use crate::{BusAdapter, Reader, Writer};
use std::cell::Cell;

#[derive(Default)]
pub struct DmaForCpuBus {
    page: Cell<Option<u8>>,
}
impl Reader for DmaForCpuBus {
    fn read(&self, _: u16) -> u8 {
        0
    }
}
impl Writer for DmaForCpuBus {
    fn write(&mut self, _: u16, data: u8) {
        self.page.set(Some(data));
    }
}
impl BusAdapter for DmaForCpuBus {
    fn address_accept(&self, addr: u16) -> bool {
        addr == 0x4014
    }
    fn take_dma_request(&self) -> Option<u8> {
        self.page.take()
    }
}
