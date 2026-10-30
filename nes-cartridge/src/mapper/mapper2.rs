use super::Mapper;
#[derive(Default)]
pub struct Mapper2 {
    bank: u8,
}
impl Mapper for Mapper2 {
    fn prg_address(&self, addr: u16, size: usize) -> usize {
        let bank = if addr < 0xc000 {
            self.bank as usize
        } else {
            size / 0x4000 - 1
        };
        bank * 0x4000 + (addr as usize & 0x3fff)
    }
    fn write_register(&mut self, _: u16, value: u8, _: u64) {
        self.bank = value;
    }
    fn reset(&mut self) {
        *self = Self::default();
    }
}
