use crate::lifter::{Value};

pub fn is_rsp(v : &Value) -> bool{
    if let Value::Reg(regnum,size) = v {
         if *regnum == 4{
             assert_eq!(size.to_u32(), 64);
             return true
        } else  {return false} 
    }
    else{false}
}

pub fn get_imm_offset(v: &Value) -> i64{
    if let Value::Imm(_,_,v) = v {*v}
    else{
        panic!("get_imm_offset called on something that is not an imm offset")
    }
}

