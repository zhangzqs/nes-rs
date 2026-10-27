use super::Mapper;
pub struct Mapper0;
impl Mapper for Mapper0 {
    fn prg_address(&self, addr: u16, _: usize) -> usize {
        (addr - 0x8000) as usize
    }
}
