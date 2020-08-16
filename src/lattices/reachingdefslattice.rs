use crate::lattices::{ConstLattice, Lattice};
use std::collections::BTreeSet;
use std::cmp::Ordering;


//TODO: unimplemented
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct ReachingDefn{
    assigned_to: u64,
    offset: u64,
}

#[derive(Eq, Clone)]
pub struct ReachingDefnLattice{
    defs: BTreeSet<ReachingDefn>
}

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
    fn meet(&self, other : Self) -> Self {
        let newdefs :  BTreeSet<ReachingDefn> =  self.defs.intersection(&other.defs).cloned().collect();
        ReachingDefnLattice {defs : newdefs}
    }
} 

impl Default for ReachingDefnLattice {
    fn default() -> Self {
        ReachingDefnLattice {defs :  BTreeSet::new()}
    }
}

//TODO: test reaching defs lattice
#[test]
fn heap_reaching_defs_test() {
    let d1 = ReachingDefn{assigned_to: 1, offset: 1};
    let d2 = ReachingDefn{assigned_to: 2, offset: 2};
    let d3 = ReachingDefn{assigned_to: 3, offset: 3};


    let bset = BTreeSet::new();
    let x1  = ReachingDefnLattice {defs : bset};
    // let x2  = ReachingDefnLattice {v : Some([])};
    // let x3  = ReachingDefnLattice {v : Some([d1.clone(), d2.clone()])};
    // let x4  = ReachingDefnLattice {v : Some(d2.clone(), d3.clone())};

    // assert_eq!(x1 == x2, false);
    // assert_eq!(x2 == x3, false);
    // assert_eq!(x3 == x4, false);

    // assert_eq!(x2 > x3, false);
    // assert_eq!(x3 > x4, false);

    // assert_eq!(x2 < x3, true);
    // assert_eq!(x3 < x4, false);

    // assert_eq!(x2.meet(x3) == ReachingDefnLattice {v : Some([])}, true);
    // assert_eq!(x3.meet(x4) == ReachingDefnLattice {v : Some([d2])}, true);
}
