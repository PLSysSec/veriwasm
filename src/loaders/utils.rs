#![allow(non_camel_case_types)]

use lucet_module::{Signature, ValueType};
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

// RDI, RSI, RDX, RCX, R8, R9,
// 7,   6,   3,   2,   8,  9,    then stack slots
pub enum ArgLoc {
    StackSlot(u32, u32), //offset, size
    Register(u32),       //register number
}

pub fn to_system_v(sig: &Signature) -> Vec<ArgLoc> {
    let mut arg_locs = Vec::new();
    let mut i_ctr = 0; // integer arg #
    let mut f_ctr = 0; // floating point arg #
    let mut stack_offset = 0;
    for arg in &sig.params {
        match arg {
            ValueType::I32 | ValueType::I64 => {
                match i_ctr {
                    0 => arg_locs.push(ArgLoc::Register(7)),
                    1 => arg_locs.push(ArgLoc::Register(6)),
                    2 => arg_locs.push(ArgLoc::Register(3)),
                    3 => arg_locs.push(ArgLoc::Register(2)),
                    4 => arg_locs.push(ArgLoc::Register(8)),
                    5 => arg_locs.push(ArgLoc::Register(9)),
                    _ => {
                        if let ValueType::I32 = arg {
                            arg_locs.push(ArgLoc::StackSlot(stack_offset, 4));
                            stack_offset += 4;
                        } else {
                            arg_locs.push(ArgLoc::StackSlot(stack_offset, 8));
                            stack_offset += 8;
                        }
                    }
                };
                i_ctr += 1;
            }
            ValueType::F32 | ValueType::F64 => {
                f_ctr += 1;
            }
        }
    }
    return arg_locs;
}
