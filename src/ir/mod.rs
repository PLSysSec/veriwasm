mod aarch64;
mod cfg;
pub mod types;
pub mod utils;
mod x64;

pub use self::cfg::fully_resolved_cfg;
// pub use self::x64::lift_cfg;
use crate::ir::types::Stmt;
use crate::loaders::types::VwArch;
use crate::VwMetadata;
use core::str::FromStr;
use crate::IRMap;
use crate::{VwModule, VW_CFG};

pub trait Liftable {
    // type Ar;
    fn lift_cfg<Ar>( &self, module: &VwModule, cfg: &VW_CFG, strict: bool) -> IRMap<Ar>;
}
// TODO: make static dispatch
impl Liftable for VwArch {
    fn lift_cfg<Ar>(&self, module: &VwModule, cfg: &VW_CFG, strict: bool) -> IRMap<Ar> {
        match self {
            VwArch::X64 => x64::lift_cfg(module, cfg, strict),
            VwArch::Aarch64 => aarch64::lift_cfg(module, cfg, strict),
        }
    }
}

// impl<Ar> VwArch {
//     fn lift_cfg(s: &str) -> Result<Self, Self::Err> {
//         match &s.to_string().to_lowercase()[..] {
//             "x86_64" => Ok(VwArch::X64),
//             "x64" => Ok(VwArch::X64),
//             "aarch64" => Ok(VwArch::Aarch64),
//             _ => Err("Unknown architecture"),
//         }
//     }
// }