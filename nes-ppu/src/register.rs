mod address;
mod control;
mod mask;
mod scroll;
mod status;

pub use address::PpuAddressRegister;
pub use control::PpuControlRegister;
pub use mask::PpuMaskRegister;
pub use scroll::PpuScrollRegister;
pub use status::PpuStatusRegister;
