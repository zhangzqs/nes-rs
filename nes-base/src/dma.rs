use crate::{BusAdapter, Reader, Writer};

pub struct DmaForCpuBus;

impl Reader for DmaForCpuBus {
    fn read(&self, addr: u16) -> u8 {
        panic!("DMA read from unsupported address: {:#04X}", addr)
    }
}

impl Writer for DmaForCpuBus {
    fn write(&mut self, _: u16, _: u8) {
        unreachable!("DMA writes must be routed through BusImpl::write to avoid borrow conflicts");
    }
}

impl BusAdapter for DmaForCpuBus {
    fn address_accept(&self, addr: u16) -> bool {
        addr == 0x4014
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DmaForCpuBus {
    pub fn write_with_bus(&self, data: u8, bus: &mut dyn BusAdapter) {
        let source_page = data as u16;
        for offset in 0..256u16 {
            let addr = (source_page << 8) | offset;
            let value = bus.read(addr);
            bus.write(0x2004, value);
        }
    }
}
