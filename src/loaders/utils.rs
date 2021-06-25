#![allow(non_camel_case_types)]

use lucet_module::{Signature, ValueType};
use std::collections::HashMap;

use crate::ir::types::ValSize;
use crate::lattices::VarIndex;
use crate::lattices::X86Regs::*;

#[derive(Clone, Debug)]
pub struct VW_Metadata {
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

// TODO: unify this with other register and stack variable slot representations
// RDI, RSI, RDX, RCX, R8, R9,
// 7,   6,   3,   2,   8,  9,    then stack slots

pub fn to_system_v(sig: &Signature) -> Vec<(VarIndex, ValSize)> {
    let mut arg_locs = Vec::new();
    let mut i_ctr = 0; // integer arg #
    let mut f_ctr = 0; // floating point arg #
    let mut stack_offset = 0;
    for arg in &sig.params {
        match arg {
            ValueType::I32 | ValueType::I64 => {
                let index = match i_ctr {
                    0 => VarIndex::Reg(Rdi),
                    1 => VarIndex::Reg(Rsi),
                    2 => VarIndex::Reg(Rdx),
                    3 => VarIndex::Reg(Rcx),
                    4 => VarIndex::Reg(R8),
                    5 => VarIndex::Reg(R9),
                    _ => {
                        if let ValueType::I32 = arg {
                            stack_offset += 4;
                        } else {
                            stack_offset += 8;
                        };
                        VarIndex::Stack(stack_offset)
                    }
                };
                i_ctr += 1;
                match arg {
                    ValueType::I32 => arg_locs.push((index, ValSize::Size32)),
                    ValueType::I64 => arg_locs.push((index, ValSize::Size64)),
                    _ => ()
                };
            }
            ValueType::F32 | ValueType::F64 => {
                f_ctr += 1;
            }
        }
    }
    return arg_locs;
}
