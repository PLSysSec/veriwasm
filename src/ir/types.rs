use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum ImmType {
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum ValSize {
    Size8,
    Size16,
    Size32,
    Size64,
    SizeOther,
}

impl ValSize {
    pub fn try_from_bits(value: u32) -> Result<Self, String> {
        match value {
            8 => Ok(ValSize::Size8),
            16 => Ok(ValSize::Size16),
            32 => Ok(ValSize::Size32),
            64 => Ok(ValSize::Size64),
            _ => Err(format!("Unknown size: {:?}", value)),
        }
    }

    pub fn into_bits(self) -> u32 {
        match self {
            ValSize::Size8 => 8,
            ValSize::Size16 => 16,
            ValSize::Size32 => 32,
            ValSize::Size64 => 64,
            ValSize::SizeOther => 64, // TODO?: panic!("unknown size? {:?}")
        }
    }

    pub fn try_from_bytes(value: u32) -> Result<Self, String> {
        match value {
            1 => Ok(ValSize::Size8),
            2 => Ok(ValSize::Size16),
            4 => Ok(ValSize::Size32),
            8 => Ok(ValSize::Size64),
            _ => Err(format!("Unknown size: {:?}", value)),
        }
    }

    pub fn into_bytes(self) -> u32 {
        match self {
            ValSize::Size8 => 1,
            ValSize::Size16 => 2,
            ValSize::Size32 => 4,
            ValSize::Size64 => 8,
            ValSize::SizeOther => 8, // TODO?: panic!("unknown size? {:?}")
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
    Reg(X86Regs, ValSize), // register mappings captured in `crate::lattices`
    Imm(ImmType, ValSize, i64), // signed, size, const
}
#[derive(Debug, Clone)]
pub enum Value {
    Mem(ValSize, MemArgs),      // mem[memargs]
    Reg(X86Regs, ValSize),
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

#[derive(Clone, Debug)]
pub enum VarIndex {
    Reg(X86Regs),
    Stack(i64)
}

#[derive(Debug, Clone)]
pub struct FunType {
    pub args: Vec<(VarIndex, ValSize)>,
    pub ret: Option<(X86Regs, ValSize)>,
}

// TODO: this should not implement PartialOrd
#[derive(PartialEq, PartialOrd, Clone, Eq, Debug, Copy, Hash)]
pub enum X86Regs {
    Rax,
    Rcx,
    Rdx,
    Rbx,
    Rsp,
    Rbp,
    Rsi,
    Rdi,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    Zf,
    Cf,
}

use self::X86Regs::*;

pub struct X86RegsIterator {
    current_reg: Option<X86Regs>
}

impl X86Regs {
    pub fn iter() -> X86RegsIterator {
        X86RegsIterator { current_reg: Some(Rax) }
    }
}

impl Iterator for X86RegsIterator {
    type Item = X86Regs;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_reg {
            None => None,
            Some(reg) => {
                match reg {
                    Rax => {
                        self.current_reg = Some(Rcx);
                        return Some(Rax);
                    }
                    Rcx => {
                        self.current_reg = Some(Rdx);
                        return Some(Rcx);
                    }
                    Rdx => {
                        self.current_reg = Some(Rbx);
                        return Some(Rdx);
                    }
                    Rbx => {
                        self.current_reg = Some(Rsp);
                        return Some(Rbx);
                    }
                    Rsp => {
                        self.current_reg = Some(Rbp);
                        return Some(Rsp);
                    }
                    Rbp => {
                        self.current_reg = Some(Rsi);
                        return Some(Rbp);
                    }
                    Rsi => {
                        self.current_reg = Some(Rdi);
                        return Some(Rsi);
                    }
                    Rdi => {
                        self.current_reg = Some(R8);
                        return Some(Rdi);
                    }
                    R8 => {
                        self.current_reg = Some(R9);
                        return Some(R8);
                    }
                    R9 => {
                        self.current_reg = Some(R10);
                        return Some(R9);
                    }
                    R10 => {
                        self.current_reg = Some(R11);
                        return Some(R10);
                    }
                    R11 => {
                        self.current_reg = Some(R12);
                        return Some(R11);
                    }
                    R12 => {
                        self.current_reg = Some(R13);
                        return Some(R12);
                    }
                    R13 => {
                        self.current_reg = Some(R14);
                        return Some(R13);
                    }
                    R14 => {
                        self.current_reg = Some(R15);
                        return Some(R14);
                    }
                    R15 => {
                        self.current_reg = Some(Zf);
                        return Some(R15);
                    }
                    Zf => {
                        self.current_reg = Some(Cf);
                        return Some(Zf);
                    }
                    Cf => {
                        self.current_reg = None;
                        return Some(Cf);
                    }
                }
            }
        }
    }
}

impl TryFrom<u8> for X86Regs {
    type Error = std::string::String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rax),
            1 => Ok(Rcx),
            2 => Ok(Rdx),
            3 => Ok(Rbx),
            4 => Ok(Rsp),
            5 => Ok(Rbp),
            6 => Ok(Rsi),
            7 => Ok(Rdi),
            8 => Ok(R8),
            9 => Ok(R9),
            10 => Ok(R10),
            11 => Ok(R11),
            12 => Ok(R12),
            13 => Ok(R13),
            14 => Ok(R14),
            15 => Ok(R15),
            16 => Ok(Zf),
            17 => Ok(Cf),
            _ => Err(format!("Unknown register: index = {:?}", value)),
        }
    }
}

impl From<X86Regs> for u8 {
    fn from(value: X86Regs) -> Self {
        value as u8
    }
}
