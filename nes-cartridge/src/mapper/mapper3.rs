use super::Mapper;
#[derive(Default)]
pub struct Mapper3 {
    chr_bank: u8,
}
impl Mapper for Mapper3 {
    fn prg_address(&self, addr: u16, _: usize) -> usize {
        (addr - 0x8000) as usize
    }
    fn chr_address(&self, addr: u16) -> usize {
        (self.chr_bank as usize & 3) * 0x2000 + addr as usize
    }
    fn write_register(&mut self, _: u16, value: u8, _: u64) {
        self.chr_bank = value;
    }
    fn reset(&mut self) {
        *self = Self::default();
    }
}
