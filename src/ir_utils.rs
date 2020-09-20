use crate::lifter::{Value,ValSize, MemArg, MemArgs};

pub fn is_rsp(v : &Value) -> bool{
    if let Value::Reg(4, ValSize::Size64) = v {
        return true
    }
    false
}

pub fn memarg_is_stack(memarg: &MemArg) -> bool {
    if let MemArg::Reg(4, regsize) = memarg{
        if let ValSize::Size64 = regsize { 
            return true
        } 
        else {
            panic!("Non 64 bit version of rsp being used")
        };
    }
    return false
}

pub fn is_stack_access(v: &Value) -> bool {
    if let Value::Mem(size, memargs) = v {
        match memargs{
            MemArgs::Mem1Arg(memarg) => 
                return memarg_is_stack(memarg),
            MemArgs::Mem2Args(memarg1, memarg2) => 
                return memarg_is_stack(memarg1) || memarg_is_stack(memarg2),
            MemArgs::Mem3Args(memarg1, memarg2, memarg3) => 
                return memarg_is_stack(memarg1) || memarg_is_stack(memarg2) || memarg_is_stack(memarg3),
            MemArgs::MemScale(memarg1, memarg2, memarg3) => 
                return memarg_is_stack(memarg1) || memarg_is_stack(memarg2) || memarg_is_stack(memarg3)
        }
    }
    false
}

pub fn extract_stack_offset(memargs: &MemArgs) -> i64{
    match memargs{
        MemArgs::Mem1Arg(memarg) => 0,
        MemArgs::Mem2Args(memarg1, memarg2) => get_imm_mem_offset(memarg2),
        MemArgs::Mem3Args(memarg1, memarg2, memarg3) | 
        MemArgs::MemScale(memarg1, memarg2, memarg3) => panic!("extract_stack_offset failed")
    }
}

pub fn is_mem_access(v: &Value) -> bool {
    if let Value::Mem(size, memargs) = v { true }
    else { false }
}

pub fn get_imm_offset(v: &Value) -> i64{
    if let Value::Imm(_,_,v) = v {*v}
    else{
        panic!("get_imm_offset called on something that is not an imm offset")
    }
}

pub fn get_imm_mem_offset(v: &MemArg) -> i64{
    if let MemArg::Imm(_,_,v) = v {*v}
    else{
        panic!("get_imm_offset called on something that is not an imm offset")
    }
}

