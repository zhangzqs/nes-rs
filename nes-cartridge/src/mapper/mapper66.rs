use super::Mapper;
#[derive(Default)]
pub struct Mapper66 {
    register: u8,
}
impl Mapper for Mapper66 {
    fn prg_address(&self, addr: u16, _: usize) -> usize {
        ((self.register as usize >> 4) & 3) * 0x8000 + (addr as usize & 0x7fff)
    }
    fn chr_address(&self, addr: u16) -> usize {
        (self.register as usize & 3) * 0x2000 + addr as usize
    }
    fn write_register(&mut self, _: u16, value: u8, _: u64) {
        self.register = value;
    }
    fn reset(&mut self) {
        *self = Self::default();
    }
}
