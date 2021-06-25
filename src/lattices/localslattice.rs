use std::convert::TryFrom;
use crate::lattices::Lattice;
use crate::lattices::reachingdefslattice::LocIdx;
pub use crate::lattices::{VariableState, VarState};


#[derive(PartialEq, Clone, Eq, Debug, Copy, Hash)]
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
}

use self::X86Regs::*;

impl TryFrom<&u8> for X86Regs {
    type Error = std::string::String;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
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
            _ => Err(format!("Unknown register: index = {:?}", value)),
        }
    }
}

impl From<X86Regs> for u8 {
    fn from(value: X86Regs) -> Self {
        value as u8
    }
}

#[derive(PartialEq, PartialOrd, Clone, Eq, Debug, Copy, Hash)]
pub enum SlotVal {
    Uninit,
    Init,
}

use self::SlotVal::*;

impl Default for SlotVal {
    fn default() -> Self {
        Uninit
    }
}

impl Lattice for SlotVal {
    fn meet(&self, other: &Self, _loc: &LocIdx) -> Self {
        match (self, other) {
            (Init, Init) => Init,
            (Uninit, _) => Uninit,
            (_, Uninit) => Uninit,
        }
    }
}

pub type LocalsLattice = VariableState<SlotVal>;
