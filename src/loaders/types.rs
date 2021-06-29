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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableType {
    Lucet,
    Wasmtime,
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
