#![allow(non_camel_case_types)]

use lucet_module::{Signature, ValueType};
use std::collections::HashMap;

use crate::ir::types::{FunType, ValSize, VarIndex, X86Regs};

use X86Regs::*;

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

// libcalls don't implicitly pass vmctx as the first argument
pub fn to_libcall(sig: &Signature) -> FunType {
    let mut arg_locs = Vec::new();
    let mut i_ctr = 0; // integer arg #
    let mut f_ctr = 0; // floating point arg #
    let mut stack_offset = 0;
    arg_locs.push((VarIndex::Reg(Rdi), ValSize::Size64));
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
                        stack_offset += 8;
                        VarIndex::Stack(stack_offset)
                    }
                };
                match arg {
                    ValueType::I32 => arg_locs.push((index, ValSize::Size32)),
                    ValueType::I64 => arg_locs.push((index, ValSize::Size64)),
                    _ => (),
                };
                i_ctr += 1;
            }
            ValueType::F32 | ValueType::F64 => {
                let index = match f_ctr {
                    0 => VarIndex::Reg(Zmm0),
                    1 => VarIndex::Reg(Zmm1),
                    2 => VarIndex::Reg(Zmm2),
                    3 => VarIndex::Reg(Zmm3),
                    4 => VarIndex::Reg(Zmm4),
                    5 => VarIndex::Reg(Zmm5),
                    6 => VarIndex::Reg(Zmm6),
                    7 => VarIndex::Reg(Zmm7),
                    _ => {
                        stack_offset += 8; //stack slots are 8 byte aligned
                        VarIndex::Stack(stack_offset)
                    }
                };
                match arg {
                    ValueType::F32 => arg_locs.push((index, ValSize::Size32)),
                    ValueType::F64 => arg_locs.push((index, ValSize::Size64)),
                    _ => (),
                };
                f_ctr += 1;
            }
        }
    }
    return FunType {
        args: arg_locs,
        ret: to_system_v_ret_ty(sig),
    };
}

// TODO: unify this with other register and stack variable slot representations
// RDI, RSI, RDX, RCX, R8, R9,
// 7,   6,   3,   2,   8,  9,    then stack slots

pub fn to_system_v(sig: &Signature) -> FunType {
    let mut arg_locs = Vec::new();
    let mut i_ctr = 0; // integer arg #
    let mut f_ctr = 0; // floating point arg #
    let mut stack_offset = 0;
    arg_locs.push((VarIndex::Reg(Rdi), ValSize::Size64));
    for arg in &sig.params {
        match arg {
            ValueType::I32 | ValueType::I64 => {
                let index = match i_ctr {
                    0 => VarIndex::Reg(Rsi),
                    1 => VarIndex::Reg(Rdx),
                    2 => VarIndex::Reg(Rcx),
                    3 => VarIndex::Reg(R8),
                    4 => VarIndex::Reg(R9),
                    _ => {
                        stack_offset += 8;
                        VarIndex::Stack(stack_offset)
                    }
                };
                match arg {
                    ValueType::I32 => arg_locs.push((index, ValSize::Size32)),
                    ValueType::I64 => arg_locs.push((index, ValSize::Size64)),
                    _ => (),
                };
                i_ctr += 1;
            }
            ValueType::F32 | ValueType::F64 => {
                let index = match f_ctr {
                    0 => VarIndex::Reg(Zmm0),
                    1 => VarIndex::Reg(Zmm1),
                    2 => VarIndex::Reg(Zmm2),
                    3 => VarIndex::Reg(Zmm3),
                    4 => VarIndex::Reg(Zmm4),
                    5 => VarIndex::Reg(Zmm5),
                    6 => VarIndex::Reg(Zmm6),
                    7 => VarIndex::Reg(Zmm7),
                    _ => {
                        stack_offset += 8; //stack slots are 8 byte aligned
                        VarIndex::Stack(stack_offset)
                    }
                };
                match arg {
                    ValueType::F32 => arg_locs.push((index, ValSize::Size32)),
                    ValueType::F64 => arg_locs.push((index, ValSize::Size64)),
                    _ => (),
                };
                f_ctr += 1;
            }
        }
    }
    return FunType {
        args: arg_locs,
        ret: to_system_v_ret_ty(sig),
    };
}

pub fn to_system_v_ret_ty(sig: &Signature) -> Option<(X86Regs, ValSize)> {
    sig.ret_ty.and_then(|ty| match ty {
        ValueType::I32 => Some((Rax, ValSize::Size32)),
        ValueType::I64 => Some((Rax, ValSize::Size64)),
        ValueType::F32 => Some((Zmm0, ValSize::Size32)),
        ValueType::F64 => Some((Zmm0, ValSize::Size64)),
    })
}
