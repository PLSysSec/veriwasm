use core::str::FromStr;
use lucet_module::Signature;
use std::collections::HashMap;
use yaxpeax_core::memory::repr::process::{
    ELFExport, ELFImport, ELFSection, ELFSymbol, ModuleData, ModuleInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VwArch {
    X64,
    Aarch64,
}

impl FromStr for VwArch {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_string().to_lowercase()[..] {
            "x86_64" => Ok(VwArch::X64),
            "x64" => Ok(VwArch::X64),
            "aarch64" => Ok(VwArch::Aarch64),
            _ => Err("Unknown architecture"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableType {
    Lucet,
    Wasmtime,
}

impl FromStr for ExecutableType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_string().to_lowercase()[..] {
            "lucet" => Ok(ExecutableType::Lucet),
            "wasmtime" => Ok(ExecutableType::Wasmtime),
            _ => Err("Unknown executable type"),
        }
    }
}

//TODO: remove public fields
pub struct VwModule {
    pub program: ModuleData,
    pub metadata: VwMetadata,
    pub format: ExecutableType,
    pub arch: VwArch,
}

#[derive(Clone, Debug)]
pub struct VwMetadata {
    pub guest_table_0: u64,
    pub lucet_tables: u64,
    pub lucet_probestack: u64,
}

#[derive(Clone, Debug)]
pub struct VwFuncInfo {
    // Index -> Type
    pub signatures: Vec<Signature>,
    // Name -> Index
    pub indexes: HashMap<String, u32>,
}
