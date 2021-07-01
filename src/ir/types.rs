use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum ImmType {
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum ValSize {
    Size1,
    Size8,
    Size16,
    Size32,
    Size64,
    Size128,
    Size256,
    Size512,
}

impl ValSize {
    pub fn try_from_bits(value: u32) -> Result<Self, String> {
        match value {
            1 => Ok(ValSize::Size1),
            8 => Ok(ValSize::Size8),
            16 => Ok(ValSize::Size16),
            32 => Ok(ValSize::Size32),
            64 => Ok(ValSize::Size64),
            128 => Ok(ValSize::Size128),
            256 => Ok(ValSize::Size256),
            512 => Ok(ValSize::Size512),
            _ => Err(format!("Not a valid bit length: {:?}", value)),
        }
    }

    pub fn into_bits(self) -> u32 {
        match self {
            ValSize::Size1 => 1,
            ValSize::Size8 => 8,
            ValSize::Size16 => 16,
            ValSize::Size32 => 32,
            ValSize::Size64 => 64,
            ValSize::Size128 => 128,
            ValSize::Size256 => 256,
            ValSize::Size512 => 512,
        }
    }

    pub fn try_from_bytes(value: u32) -> Result<Self, String> {
        match value {
            1 => Ok(ValSize::Size8),
            2 => Ok(ValSize::Size16),
            4 => Ok(ValSize::Size32),
            8 => Ok(ValSize::Size64),
            16 => Ok(ValSize::Size128),
            32 => Ok(ValSize::Size256),
            64 => Ok(ValSize::Size512),
            _ => Err(format!("Not a valid byte length: {:?}", value)),
        }
    }

    pub fn into_bytes(self) -> u32 {
        match self {
            ValSize::Size1 => panic!("1 bit flag cannot be converted to bytes"),
            ValSize::Size8 => 1,
            ValSize::Size16 => 2,
            ValSize::Size32 => 4,
            ValSize::Size64 => 8,
            ValSize::Size128 => 16,
            ValSize::Size256 => 32,
            ValSize::Size512 => 64,
        }
    }

    pub fn fp_offset() -> u8 {
        u8::from(Zmm0)
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
    Reg(X86Regs, ValSize),      // register mappings captured in `crate::lattices`
    Imm(ImmType, ValSize, i64), // signed, size, const
}
#[derive(Debug, Clone)]
pub enum Value {
    Mem(ValSize, MemArgs), // mem[memargs]
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
    Stack(i64),
}

#[derive(Debug, Clone)]
pub struct FunType {
    pub args: Vec<(VarIndex, ValSize)>,
    pub ret: Option<(X86Regs, ValSize)>,
}

// TODO: this should not implement PartialOrd
// TODO: add flags iter
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
    Pf,
    Sf,
    Of,
    Zmm0,
    Zmm1,
    Zmm2,
    Zmm3,
    Zmm4,
    Zmm5,
    Zmm6,
    Zmm7,
    Zmm8,
    Zmm9,
    Zmm10,
    Zmm11,
    Zmm12,
    Zmm13,
    Zmm14,
    Zmm15,
}

use self::X86Regs::*;

impl X86Regs {
    pub fn is_flag(self) -> bool {
        match self {
            Zf | Cf | Pf | Sf | Of => true,
            _ => false,
        }
    }
}

pub struct X86RegsIterator {
    current_reg: Option<X86Regs>,
}

impl X86Regs {
    pub fn iter() -> X86RegsIterator {
        X86RegsIterator {
            current_reg: Some(Rax),
        }
    }
}

impl Iterator for X86RegsIterator {
    type Item = X86Regs;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_reg {
            None => None,
            Some(reg) => match reg {
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
                    self.current_reg = Some(Pf);
                    return Some(Cf);
                }
                Pf => {
                    self.current_reg = Some(Sf);
                    return Some(Pf);
                }
                Sf => {
                    self.current_reg = Some(Of);
                    return Some(Sf);
                }
                Of => {
                    self.current_reg = Some(Zmm0);
                    return Some(Of);
                }
                Zmm0 => {
                    self.current_reg = Some(Zmm1);
                    return Some(Zmm0);
                }
                Zmm1 => {
                    self.current_reg = Some(Zmm2);
                    return Some(Zmm1);
                }
                Zmm2 => {
                    self.current_reg = Some(Zmm3);
                    return Some(Zmm2);
                }
                Zmm3 => {
                    self.current_reg = Some(Zmm4);
                    return Some(Zmm3);
                }
                Zmm4 => {
                    self.current_reg = Some(Zmm5);
                    return Some(Zmm4);
                }
                Zmm5 => {
                    self.current_reg = Some(Zmm6);
                    return Some(Zmm5);
                }
                Zmm6 => {
                    self.current_reg = Some(Zmm7);
                    return Some(Zmm6);
                }
                Zmm7 => {
                    self.current_reg = Some(Zmm8);
                    return Some(Zmm7);
                }
                Zmm8 => {
                    self.current_reg = Some(Zmm9);
                    return Some(Zmm8);
                }
                Zmm9 => {
                    self.current_reg = Some(Zmm10);
                    return Some(Zmm9);
                }
                Zmm10 => {
                    self.current_reg = Some(Zmm11);
                    return Some(Zmm10);
                }
                Zmm11 => {
                    self.current_reg = Some(Zmm12);
                    return Some(Zmm11);
                }
                Zmm12 => {
                    self.current_reg = Some(Zmm13);
                    return Some(Zmm12);
                }
                Zmm13 => {
                    self.current_reg = Some(Zmm14);
                    return Some(Zmm13);
                }
                Zmm14 => {
                    self.current_reg = Some(Zmm15);
                    return Some(Zmm14);
                }
                Zmm15 => {
                    self.current_reg = None;
                    return Some(Zmm15);
                }
            },
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
            18 => Ok(Pf),
            19 => Ok(Sf),
            20 => Ok(Of),
            21 => Ok(Zmm0),
            22 => Ok(Zmm1),
            23 => Ok(Zmm2),
            24 => Ok(Zmm3),
            25 => Ok(Zmm4),
            26 => Ok(Zmm5),
            27 => Ok(Zmm6),
            28 => Ok(Zmm7),
            29 => Ok(Zmm8),
            30 => Ok(Zmm9),
            31 => Ok(Zmm10),
            32 => Ok(Zmm11),
            33 => Ok(Zmm12),
            34 => Ok(Zmm13),
            35 => Ok(Zmm14),
            36 => Ok(Zmm15),
            _ => Err(format!("Unknown register: index = {:?}", value)),
        }
    }
}

impl From<X86Regs> for u8 {
    fn from(value: X86Regs) -> Self {
        value as u8
    }
}
