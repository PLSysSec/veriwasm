use crate::utils::lifter::{MemArg, MemArgs, ValSize, Value, Stmt, IRMap};

pub fn is_rsp(v: &Value) -> bool {
    match v {
        Value::Reg(4, ValSize::Size64) => return true,
        Value::Reg(4, ValSize::Size32)
        | Value::Reg(4, ValSize::Size16)
        | Value::Reg(4, ValSize::Size8) => panic!("Illegal RSP access"),
        _ => return false,
    }
}

pub fn is_zf(v: &Value) -> bool {
    match v {
        Value::Reg(16, _) => return true,
        _ => return false,
    }
}

pub fn is_irrelevant_reg(v: &Value) -> bool {
    if let Value::Reg(_, ValSize::SizeOther) = v {
        return true;
    }
    false
}

pub fn memarg_is_stack(memarg: &MemArg) -> bool {
    if let MemArg::Reg(4, regsize) = memarg {
        if let ValSize::Size64 = regsize {
            return true;
        } else {
            panic!("Non 64 bit version of rsp being used")
        };
    }
    return false;
}

pub fn is_stack_access(v: &Value) -> bool {
    if let Value::Mem(_size, memargs) = v {
        match memargs {
            MemArgs::Mem1Arg(memarg) => return memarg_is_stack(memarg),
            MemArgs::Mem2Args(memarg1, memarg2) => {
                return memarg_is_stack(memarg1) || memarg_is_stack(memarg2)
            }
            MemArgs::Mem3Args(memarg1, memarg2, memarg3) => {
                return memarg_is_stack(memarg1)
                    || memarg_is_stack(memarg2)
                    || memarg_is_stack(memarg3)
            }
            MemArgs::MemScale(memarg1, memarg2, memarg3) => {
                return memarg_is_stack(memarg1)
                    || memarg_is_stack(memarg2)
                    || memarg_is_stack(memarg3)
            }
        }
    }
    false
}

pub fn extract_stack_offset(memargs: &MemArgs) -> i64 {
    match memargs {
        MemArgs::Mem1Arg(_memarg) => 0,
        MemArgs::Mem2Args(_memarg1, memarg2) => get_imm_mem_offset(memarg2),
        MemArgs::Mem3Args(_memarg1, _memarg2, _memarg3)
        | MemArgs::MemScale(_memarg1, _memarg2, _memarg3) => panic!("extract_stack_offset failed"),
    }
}

pub fn is_mem_access(v: &Value) -> bool {
    if let Value::Mem(_, _) = v {
        true
    } else {
        false
    }
}

pub fn get_imm_offset(v: &Value) -> i64 {
    if let Value::Imm(_, _, v) = v {
        *v
    } else {
        panic!("get_imm_offset called on something that is not an imm offset")
    }
}

pub fn get_imm_mem_offset(v: &MemArg) -> i64 {
    if let MemArg::Imm(_, _, v) = v {
        *v
    } else {
        panic!("get_imm_offset called on something that is not an imm offset")
    }
}

pub fn has_indirect_calls(irmap: &IRMap) -> bool {
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

pub fn has_indirect_jumps(irmap: &IRMap) -> bool {
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
