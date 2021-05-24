use crate::lattices::{Lattice, VariableState};
use std::cmp::Ordering;
use std::collections::BTreeSet;

#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Debug)]
pub struct LocIdx {
    pub addr: u64,
    pub idx: u32,
}

#[derive(Eq, Clone, Debug)]
pub struct ReachingDefnLattice {
    pub defs: BTreeSet<LocIdx>,
}

pub type ReachLattice = VariableState<ReachingDefnLattice>;

impl ReachingDefnLattice {
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }
}

impl PartialOrd for ReachingDefnLattice {
    fn partial_cmp(&self, other: &ReachingDefnLattice) -> Option<Ordering> {
        if &self.defs == &other.defs {
            return Some(Ordering::Equal);
        } else if self.defs.is_subset(&other.defs) {
            return Some(Ordering::Greater);
        } else if other.defs.is_subset(&self.defs) {
            return Some(Ordering::Less);
        } else {
            return None;
        }
    }
}

impl PartialEq for ReachingDefnLattice {
    fn eq(&self, other: &ReachingDefnLattice) -> bool {
        self.defs == other.defs
    }
}

impl Lattice for ReachingDefnLattice {
    fn meet(&self, other: &Self, _loc_idx: &LocIdx) -> Self {
        let newdefs: BTreeSet<LocIdx> = self.defs.union(&other.defs).cloned().collect();
        ReachingDefnLattice { defs: newdefs }
    }
}

impl Default for ReachingDefnLattice {
    fn default() -> Self {
        ReachingDefnLattice {
            defs: BTreeSet::new(),
        }
    }
}

pub fn singleton(loc_idx: LocIdx) -> ReachingDefnLattice {
    let mut bset = BTreeSet::new();
    bset.insert(loc_idx);
    ReachingDefnLattice { defs: bset }
}

pub fn loc(addr: u64, idx: u32) -> ReachingDefnLattice {
    singleton(LocIdx {
        addr: addr,
        idx: idx,
    })
}
