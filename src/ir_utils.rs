use crate::lifter::{Value,ValSize};

pub fn is_rsp(v : &Value) -> bool{
    if let Value::Reg(4, ValSize::Size64) = v {
        return true
    }
    false
}

pub fn get_imm_offset(v: &Value) -> i64{
    if let Value::Imm(_,_,v) = v {*v}
    else{
        panic!("get_imm_offset called on something that is not an imm offset")
    }
}

