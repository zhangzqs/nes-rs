use super::Mapper;
use nes_base::Mirroring;
#[derive(Default)]
pub struct Mapper7 {
    register: u8,
}
impl Mapper for Mapper7 {
    fn prg_address(&self, addr: u16, _: usize) -> usize {
        (self.register as usize & 7) * 0x8000 + (addr as usize & 0x7fff)
    }
    fn write_register(&mut self, _: u16, value: u8, _: u64) {
        self.register = value;
    }
    fn mirroring(&self) -> Option<Mirroring> {
        Some(if self.register & 0x10 == 0 {
            Mirroring::SingleScreen
        } else {
            Mirroring::SingleScreenHigh
        })
    }
    fn reset(&mut self) {
        *self = Self::default();
    }
}
