#![allow(non_camel_case_types)]

#[derive(Clone, Debug)]
pub struct VW_Metadata {
    pub guest_table_0: u64,
    pub lucet_tables: u64,
    pub lucet_probestack: u64,
}

#[derive(Clone, Debug)]
pub struct FuncSignature {
    pub ty: u64,
}

pub type FuncSignatures = Vec<FuncSignature>;


// pub fn read_module_buffer(program: &ModuleData) -> Option<Vec<u8>> {

// }
