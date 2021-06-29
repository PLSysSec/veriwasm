mod aarch64;
mod cfg;
pub mod types;
pub mod utils;
mod x64;

pub use self::cfg::fully_resolved_cfg;
pub use self::x64::lift_cfg;
use crate::ir::types::Stmt;
use crate::loaders::types::VwArch;
use crate::VwMetadata;
use core::str::FromStr;

pub trait Liftable {
    fn lift(
        &self,
        instr: &yaxpeax_x86::long_mode::Instruction,
        addr: &u64,
        metadata: &VwMetadata,
        strict: bool,
    ) -> Vec<Stmt>;
}
// TODO: make static dispatch
impl Liftable for VwArch {
    fn lift(
        &self,
        instr: &yaxpeax_x86::long_mode::Instruction,
        addr: &u64,
        metadata: &VwMetadata,
        strict: bool,
    ) -> Vec<Stmt> {
        match self {
            VwArch::X64 => x64::lift(instr, addr, metadata, strict),
            VwArch::Aarch64 => aarch64::lift(instr, addr, metadata, strict),
        }
    }
}
