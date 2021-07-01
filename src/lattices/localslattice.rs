use crate::ir::types::X86Regs;
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::Lattice;
pub use crate::lattices::{VarState, VariableState};
use std::cmp::Ordering;
use std::convert::TryFrom;

use Ordering::*;
use X86Regs::*;

#[derive(PartialEq, Clone, Eq, Debug, Copy, Hash)]
pub enum SlotVal {
    Uninit,
    Init,
    InitialRegVal(X86Regs),
}

impl PartialOrd for SlotVal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Uninit, Uninit) => Some(Equal),
            (Uninit, _) => Some(Less),
            (_, Uninit) => Some(Greater),
            (Init, Init) => Some(Equal),
            (Init, _) => Some(Greater),
            (_, Init) => Some(Less),
            (InitialRegVal(r1), InitialRegVal(r2)) => {
                if r1 == r1 {
                    Some(Equal)
                } else {
                    None
                }
            }
        }
    }
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
            (Init, InitialRegVal(_)) => Uninit,
            (InitialRegVal(_), Init) => Uninit,
            (InitialRegVal(r1), InitialRegVal(r2)) => {
                if r1 == r2 {
                    InitialRegVal(*r1)
                } else {
                    Uninit
                }
            }
        }
    }
}

pub type LocalsLattice = VariableState<SlotVal>;
