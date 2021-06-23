#![allow(non_camel_case_types)]

use lucet_module::Signature;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct VW_Metadata {
    pub guest_table_0: u64,
    pub lucet_tables: u64,
    pub lucet_probestack: u64,
}

#[derive(Debug)]
pub struct VwFuncInfo {
    // Index -> Type
    pub signatures: Vec<Signature>,
    // Name -> Index
    pub indexes: HashMap<String, u32>,
}
