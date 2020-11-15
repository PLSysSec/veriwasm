use crate::lattices::{Lattice, VariableState};
use std::collections::BTreeSet;
use std::cmp::Ordering;

#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Debug)]
pub struct LocIdx{
    pub addr: u64,
    pub idx : u32
}

#[derive(Eq, Clone, Debug)]
pub struct ReachingDefnLattice{
    defs: BTreeSet<LocIdx>
}

pub type ReachLattice =  VariableState<ReachingDefnLattice>;

impl PartialOrd for ReachingDefnLattice {
    fn partial_cmp(&self, other: &ReachingDefnLattice) -> Option<Ordering> {
        if &self.defs == &other.defs {
            return Some(Ordering::Greater)
        }
        else if self.defs.is_subset(&other.defs){
            return Some(Ordering::Less)
        }
        else if other.defs.is_subset(&self.defs){
            return Some(Ordering::Greater)
        }
        else{
            return None
        }
    }
}

impl PartialEq for ReachingDefnLattice {
    fn eq(&self, other: &ReachingDefnLattice) -> bool {
        self.defs == other.defs
    }
}

impl Lattice for ReachingDefnLattice {
    fn meet(&self, other : &Self) -> Self {
        let newdefs :  BTreeSet<LocIdx> =  self.defs.intersection(&other.defs).cloned().collect();
        ReachingDefnLattice {defs : newdefs}
    }
} 

impl Default for ReachingDefnLattice {
    fn default() -> Self {
        ReachingDefnLattice {defs :  BTreeSet::new()}
    }
}

pub fn singleton(loc_idx : LocIdx) -> ReachingDefnLattice{
    let mut bset = BTreeSet::new();
    bset.insert(loc_idx);
    ReachingDefnLattice{defs: bset}
}


#[test]
fn heap_reaching_defs_test() {
    let d1 = LocIdx{addr: 1, idx : 1};
    let d2 = LocIdx{addr: 2, idx : 2};
    let d3 = LocIdx{addr: 3, idx : 3};
    let d4 = LocIdx{addr: 4, idx : 4};


    let mut bset1 = BTreeSet::new();
    bset1.insert(d1);
    bset1.insert(d2);
    let x1  = ReachingDefnLattice {defs : bset1};

    let mut bset2 = BTreeSet::new();
    bset2.insert(d3);
    bset2.insert(d4);
    let x2  = ReachingDefnLattice {defs : bset2};

    assert_eq!(x1 == x2, false);
    assert_eq!(x1 > x2, false);
    assert_eq!(x1 < x2, false);
    assert_eq!(x1 >= x2, false);
    assert_eq!(x1 <= x2, false);
    assert_eq!(x1.meet(&x2) == Default::default(), true);
}
