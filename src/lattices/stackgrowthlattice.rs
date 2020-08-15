use std::cmp::Ordering;
use crate::lattices::{Lattice, ConstLattice};

type StackGrowthLattice = ConstLattice<u64>;


#[test]
fn stack_growth_lattice_test() {
    let x1  = StackGrowthLattice {v : None};
    let x2  = StackGrowthLattice {v : Some(1)};
    let x3  = StackGrowthLattice {v : Some(1)};
    let x4  = StackGrowthLattice {v : Some(2)};


    assert_eq!(x1 == x2, false);
    assert_eq!(x2 == x3, true);
    assert_eq!(x3 == x4, false);

    assert_eq!(x1 != x2, true);
    assert_eq!(x2 != x3, false);
    assert_eq!(x3 != x4, true);

    assert_eq!(x1 > x2, false);
    assert_eq!(x2 > x3, false);
    assert_eq!(x3 > x4, false);

    assert_eq!(x1 < x2, true);
    assert_eq!(x2 < x3, false);
    assert_eq!(x3 < x4, false);

    assert_eq!(x1.meet(x2) == StackGrowthLattice {v : None}, true);
    assert_eq!(x2.meet(x3) == StackGrowthLattice {v : Some(1)}, true);
    assert_eq!(x3.meet(x4) == StackGrowthLattice {v : None}, true);
}
