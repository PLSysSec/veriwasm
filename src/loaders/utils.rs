#![allow(non_camel_case_types)]

#[derive(Clone)]
pub struct VW_Metadata {
    pub guest_table_0: u64,
    pub lucet_tables: u64,
    pub lucet_probestack: u64,
}
