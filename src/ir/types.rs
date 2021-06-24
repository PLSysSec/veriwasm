use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ImmType {
    Signed,
    Unsigned,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValSize {
    Size8,
    Size16,
    Size32,
    Size64,
    SizeOther,
}

impl ValSize {
    pub fn to_u32(&self) -> u32 {
        match self {
            ValSize::Size8 => 8,
            ValSize::Size16 => 16,
            ValSize::Size32 => 32,
            ValSize::Size64 => 64,
            ValSize::SizeOther => 64, //panic!("unknown size? {:?}")
        }
    }
}

#[derive(Debug, Clone)]
pub enum MemArgs {
    Mem1Arg(MemArg),                  // [arg]
    Mem2Args(MemArg, MemArg),         // [arg1 + arg2]
    Mem3Args(MemArg, MemArg, MemArg), // [arg1 + arg2 + arg3]
    MemScale(MemArg, MemArg, MemArg), // [arg1 + arg2 * arg3]
}
#[derive(Debug, Clone)]
pub enum MemArg {
    Reg(u8, ValSize), // register mappings captured in `crate::lattices::regslattice`
    Imm(ImmType, ValSize, i64), // signed, size, const
}
#[derive(Debug, Clone)]
pub enum Value {
    Mem(ValSize, MemArgs),      // mem[memargs]
    Reg(u8, ValSize),           // register mappings captured in `crate::lattices::regslattice`
    Imm(ImmType, ValSize, i64), // signed, size, const
    RIPConst,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Clear(Value, Vec<Value>),                      // clear v <- vs
    Unop(Unopcode, Value, Value),                  // v1 <- uop v2
    Binop(Binopcode, Value, Value, Value),         // v1 <- bop v2 v3
    Undefined,                                     // undefined
    Ret,                                           // return
    Branch(yaxpeax_x86::long_mode::Opcode, Value), // br branch-type v
    Call(Value),                                   // call v
    ProbeStack(u64),                               // probestack
}

#[derive(Debug, Clone)]
pub enum Unopcode {
    Mov,
    Movsx,
}
#[derive(Debug, Clone)]
pub enum Binopcode {
    Test,
    Rol,
    Cmp,
    Shl,
    And,
    Add,
    Sub,
}

pub type IRBlock = Vec<(u64, Vec<Stmt>)>;
pub type IRMap = HashMap<u64, IRBlock>;
