use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

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

use ValSize::*;

impl ValSize {
    pub fn try_from_bits(value: u32) -> Result<Self, String> {
        match value {
            1 => Ok(Size1),
            8 => Ok(Size8),
            16 => Ok(Size16),
            32 => Ok(Size32),
            64 => Ok(Size64),
            128 => Ok(Size128),
            256 => Ok(Size256),
            512 => Ok(Size512),
            _ => Err(format!("Not a valid bit length: {:?}", value)),
        }
    }

    pub fn into_bits(self) -> u32 {
        match self {
            Size1 => 1,
            Size8 => 8,
            Size16 => 16,
            Size32 => 32,
            Size64 => 64,
            Size128 => 128,
            Size256 => 256,
            Size512 => 512,
        }
    }

    pub fn try_from_bytes(value: u32) -> Result<Self, String> {
        match value {
            1 => Ok(Size8),
            2 => Ok(Size16),
            4 => Ok(Size32),
            8 => Ok(Size64),
            16 => Ok(Size128),
            32 => Ok(Size256),
            64 => Ok(Size512),
            _ => Err(format!("Not a valid byte length: {:?}", value)),
        }
    }

    pub fn into_bytes(self) -> u32 {
        match self {
            Size1 => panic!("1 bit flag cannot be converted to bytes"),
            Size8 => 1,
            Size16 => 2,
            Size32 => 4,
            Size64 => 8,
            Size128 => 16,
            Size256 => 32,
            Size512 => 64,
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

impl MemArgs {
    fn add_imm(&self, imm: i64) -> Self {
        match self {
            MemArgs::Mem1Arg(arg) => {
                MemArgs::Mem2Args(arg.clone(), MemArg::Imm(ImmType::Signed, Size64, imm))
            }
            _ => panic!("adding to bad memargs"),
        }
    }

    pub fn extract_stack_offset(&self) -> i64 {
        match self {
            MemArgs::Mem1Arg(_memarg) => 0,
            MemArgs::Mem2Args(_memarg1, memarg2) => memarg2.to_imm(),
            MemArgs::Mem3Args(_memarg1, _memarg2, _memarg3)
            | MemArgs::MemScale(_memarg1, _memarg2, _memarg3) => {
                panic!("extract_stack_offset failed")
            }
        }
    }
}

impl TryFrom<Value> for MemArg {
    type Error = &'static str;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Reg(r, sz) => Ok(Self::Reg(r, sz)),
            Value::Mem(_, _) => Err("Memargs cannot be nested"),
            Value::Imm(ty, imm, sz) => Ok(Self::Imm(ty, imm, sz)),
            Value::RIPConst => Err("Memargs cannot be made from RIPConst"),
        }
    }
}

impl MemArg {
    pub fn is_imm(&self) -> bool {
        matches!(self, Self::Imm(_, _, _))
    }

    pub fn is_reg(&self) -> bool {
        matches!(self, Self::Reg(_, _))
    }

    pub fn is_rsp(&self) -> bool {
        match self {
            Self::Reg(r, Size64) if r.is_rsp() => true,
            Self::Reg(r, _) if r.is_rsp() => panic!("Illegal RSP access"),
            _ => false,
        }
    }

    pub fn is_rbp(&self) -> bool {
        match self {
            Self::Reg(r, Size64) if r.is_rbp() => true,
            Self::Reg(r, _) if r.is_rbp() => panic!("Illegal RBP access"),
            _ => false,
        }
    }

    pub fn to_imm(&self) -> i64 {
        match self {
            Self::Imm(_, _, v) => return *v,
            _ => panic!("Not an imm"),
        }
    }

    pub fn to_reg(&self) -> X86Regs {
        match self {
            Self::Reg(r, _) => *r,
            _ => panic!("That's not a reg!"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Mem(ValSize, MemArgs), // mem[memargs]
    Reg(X86Regs, ValSize),
    Imm(ImmType, ValSize, i64), // signed, size, const
    RIPConst,
}

impl Value {
    pub fn is_mem(&self) -> bool {
        matches!(self, Self::Mem(_, _))
    }

    pub fn is_imm(&self) -> bool {
        matches!(self, Self::Imm(_, _, _))
    }

    pub fn is_reg(&self) -> bool {
        matches!(self, Self::Reg(_, _))
    }

    pub fn get_size(&self) -> ValSize {
        match self {
            Self::Mem(sz, _) | Self::Reg(_, sz) | Self::Imm(_, sz, _) => *sz,
            Self::RIPConst => Size64,
        }
    }

    pub fn to_reg(&self) -> X86Regs {
        match self {
            Self::Reg(r, _) => *r,
            _ => panic!("That's not a reg!"),
        }
    }

    pub fn to_mem(&self) -> MemArgs {
        match self {
            Self::Mem(_, memargs) => memargs.clone(),
            _ => panic!("That's not a reg!"),
        }
    }

    pub fn is_rsp(&self) -> bool {
        match self {
            Self::Reg(r, Size64) if r.is_rsp() => true,
            Self::Reg(r, _) if r.is_rsp() => panic!("Illegal RSP access"),
            _ => false,
        }
    }

    pub fn is_rbp(&self) -> bool {
        match self {
            Self::Reg(r, Size64) if r.is_rbp() => true,
            Self::Reg(r, _) if r.is_rbp() => panic!("Illegal RSP access"),
            _ => false,
        }
    }

    pub fn is_zf(&self) -> bool {
        match self {
            Self::Reg(r, _) if r.is_zf() => return true,
            _ => return false,
        }
    }

    pub fn as_imm_val(&self) -> i64 {
        if let Value::Imm(_, _, v) = self {
            *v
        } else {
            panic!("as_imm_val called on something that is not an imm offset")
        }
    }

    pub fn add_imm(&self, imm: i64) -> Self {
        match self {
            Self::Mem(sz, memargs) => Self::Mem(*sz, memargs.add_imm(imm)),
            _ => panic!("adding to bad value"),
        }
    }

    pub fn is_stack_access(&self) -> bool {
        if let Value::Mem(_size, memargs) = self {
            use MemArgs::*;
            match memargs {
                Mem1Arg(memarg) => return memarg.is_rsp(),
                Mem2Args(memarg1, memarg2) => return memarg1.is_rsp() || memarg2.is_rsp(),
                Mem3Args(memarg1, memarg2, memarg3) | MemScale(memarg1, memarg2, memarg3) => {
                    return memarg1.is_rsp() || memarg2.is_rsp() || memarg3.is_rsp()
                } // MemArgs::MemScale(memarg1, memarg2, memarg3) => {
                  //     return memarg1.is_rsp()
                  //         || memarg2.is_rsp()
                  //         || memarg3.is_rsp()
                  // }
            }
        }
        false
    }

    pub fn is_frame_access(&self) -> bool {
        use MemArgs::*;
        if let Value::Mem(_size, memargs) = self {
            match memargs {
                Mem1Arg(memarg) => return memarg.is_rbp(),
                Mem2Args(memarg1, memarg2) => return memarg1.is_rbp() || memarg2.is_rbp(),
                Mem3Args(memarg1, memarg2, memarg3) | MemScale(memarg1, memarg2, memarg3) => {
                    return memarg1.is_rbp() || memarg2.is_rbp() || memarg3.is_rbp()
                } // MemArgs::MemScale(memarg1, memarg2, memarg3) => {
                  //     return memarg1.is_rbp()
                  //         || memarg2.is_rbp()
                  //         || memarg3.is_rbp()
                  // }
            }
        }
        false
    }
}

impl From<i64> for Value {
    fn from(num: i64) -> Self {
        Self::Imm(ImmType::Signed, Size64, num)
    }
}

impl From<MemArg> for Value {
    fn from(arg: MemArg) -> Self {
        match arg {
            MemArg::Reg(r, sz) => Self::Reg(r, sz),
            MemArg::Imm(ty, imm, sz) => Self::Imm(ty, imm, sz),
        }
    }
}

// Parameterized by architecture register set
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

//Phantom data denotes what type of register we are iterating over,
// This allows us to use same Iterator and struct impl for all types of registers
// without having an unused type parameter
pub struct RegsIterator {
    current_reg: u8,
    //reg_type: PhantomData,
}

impl Iterator for RegsIterator {
    type Item = X86Regs;

    fn next(&mut self) -> Option<Self::Item> {
        let next_reg = self.current_reg + 1;
        match Self::Item::try_from(next_reg) {
            Ok(r) => {
                let current = self.current_reg;
                self.current_reg = next_reg;
                Some(r)
            }
            Err(_) => None,
        }
    }
}

impl RegT for X86Regs {
    fn is_rsp(&self) -> bool {
        self == &Rsp
    }

    fn is_rbp(&self) -> bool {
        self == &Rbp
    }

    fn is_zf(&self) -> bool {
        self == &Zf
    }

    fn pinned_heap_reg() -> Self {
        Rdi
    }

    fn pinned_vmctx_reg() -> Self {
        Rdi
    }

    fn is_caller_saved(&self) -> bool {
        match self {
            Rax | Rcx | Rdx | Rsi | Rdi | R8 | R9 | R10 | R11 | Zf | Cf | Pf | Sf | Of => true,
            _ => false,
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

pub enum ParseErr<E> {
    Incomplete, // input too short
    Error(E),   // recoverable
    Failure(E), // unrecoverable
}

//#[derive(PartialEq, PartialOrd, Clone, Eq, Debug, Copy, Hash)]
pub trait RegT:
    Debug + Clone + PartialEq + Eq + PartialOrd + Hash + Copy + TryFrom<u8> + Into<u8>
{
    fn is_rsp(&self) -> bool;
    fn is_rbp(&self) -> bool;
    fn is_zf(&self) -> bool;
    fn pinned_heap_reg() -> Self;
    fn pinned_vmctx_reg() -> Self;
    fn is_caller_saved(&self) -> bool;
    fn iter() -> RegsIterator {
        RegsIterator {
            current_reg: 0,
            //reg_type: PhantomData,
        }
    }
}

pub fn valsize(num: u32) -> ValSize {
    ValSize::try_from_bits(num).unwrap()
}

pub fn mk_value_i64(num: i64) -> Value {
    Value::Imm(ImmType::Signed, Size64, num)
}

pub fn get_rsp_offset(memargs: &MemArgs) -> Option<i64> {
    match memargs {
        MemArgs::Mem1Arg(arg) if arg.is_rsp() => {
            return Some(0);
        }
        MemArgs::Mem2Args(arg1, arg2) if arg1.is_rsp() => {
            if let MemArg::Imm(_, _, offset) = arg2 {
                return Some(*offset);
            }
            None
        }
        _ => None,
    }
}
