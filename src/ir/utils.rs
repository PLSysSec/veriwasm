use crate::ir::types::*;

use ValSize::*;

pub fn is_stack_access<Ar: RegT>(v: &Value<Ar>) -> bool {
    if let Value::Mem(_size, memargs) = v {
        match memargs {
            MemArgs::Mem1Arg(memarg) => return memarg.is_rsp(),
            MemArgs::Mem2Args(memarg1, memarg2) => return memarg1.is_rsp() || memarg2.is_rsp(),
            MemArgs::Mem3Args(memarg1, memarg2, memarg3) => {
                return memarg1.is_rsp() || memarg2.is_rsp() || memarg3.is_rsp()
            }
            MemArgs::MemScale(memarg1, memarg2, memarg3) => {
                return memarg1.is_rsp() || memarg2.is_rsp() || memarg3.is_rsp()
            }
        }
    }
    false
}

pub fn is_bp_access<Ar: RegT>(v: &Value<Ar>) -> bool {
    if let Value::Mem(_size, memargs) = v {
        match memargs {
            MemArgs::Mem1Arg(memarg) => return memarg.is_rbp(),
            MemArgs::Mem2Args(memarg1, memarg2) => return memarg1.is_rbp() || memarg2.is_rbp(),
            MemArgs::Mem3Args(memarg1, memarg2, memarg3) => {
                return memarg1.is_rbp() || memarg2.is_rbp() || memarg3.is_rbp()
            }
            MemArgs::MemScale(memarg1, memarg2, memarg3) => {
                return memarg1.is_rbp() || memarg2.is_rbp() || memarg3.is_rbp()
            }
        }
    }
    false
}

/// Precondition: should only be called on stack accesses
/// mem[rsp] => 0
/// mem[rsp + c] => c
pub fn extract_stack_offset<Ar: RegT>(memargs: &MemArgs<Ar>) -> i64 {
    match memargs {
        MemArgs::Mem1Arg(_memarg) => 0,
        MemArgs::Mem2Args(_memarg1, memarg2) => memarg2.to_imm(),
        _ => panic!("extract_stack_offset failed"),
    }
}

// pub fn is_mem_access<Ar>(v: &Value<Ar>) -> bool {
//     if let Value::Mem(_, _) = v {
//         true
//     } else {
//         false
//     }
// }

// pub fn get_imm_offset<Ar>(v: &Value<Ar>) -> i64 {
//     if let Value::Imm(_, _, v) = v {
//         *v
//     } else {
//         panic!("get_imm_offset called on something that is not an imm offset")
//     }
// }

// pub fn get_imm_mem_offset<Ar>(v: &MemArg<Ar>) -> i64 {
//     if let MemArg::Imm(_, _, v) = v {
//         *v
//     } else {
//         panic!("get_imm_offset called on something that is not an imm offset")
//     }
// }

pub fn has_indirect_calls<Ar>(irmap: &IRMap<Ar>) -> bool {
    for (_block_addr, ir_block) in irmap {
        for (_addr, ir_stmts) in ir_block {
            for (_idx, ir_stmt) in ir_stmts.iter().enumerate() {
                match ir_stmt {
                    Stmt::Call(Value::Reg(_, _)) | Stmt::Call(Value::Mem(_, _)) => return true,
                    _ => (),
                }
            }
        }
    }
    false
}

pub fn has_indirect_jumps<Ar>(irmap: &IRMap<Ar>) -> bool {
    for (_block_addr, ir_block) in irmap {
        for (_addr, ir_stmts) in ir_block {
            for (_idx, ir_stmt) in ir_stmts.iter().enumerate() {
                match ir_stmt {
                    Stmt::Branch(_, Value::Reg(_, _)) | Stmt::Branch(_, Value::Mem(_, _)) => {
                        return true
                    }
                    _ => (),
                }
            }
        }
    }
    false
}

pub fn get_rsp_offset<Ar: RegT>(memargs: &MemArgs<Ar>) -> Option<i64> {
    match memargs {
        MemArgs::Mem1Arg(arg) if arg.is_rsp() => Some(0),
        MemArgs::Mem2Args(arg1, arg2) if arg1.is_rsp() => {
            if let MemArg::Imm(_, _, offset) = arg2 {
                return Some(*offset);
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn valsize(num: u32) -> ValSize {
    ValSize::try_from_bits(num).unwrap()
}

pub fn mk_value_i64<Ar>(num: i64) -> Value<Ar> {
    Value::Imm(ImmType::Signed, Size64, num)
}
