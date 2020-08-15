use crate::lattices::{ConstLattice, Lattice};
use crate::lattices::davlattice::DAVLattice

#[derive(Clone, Copy, PartialEq, Eq)]
enum CallCheckValue {
    GuestTableBase,
    LucetTablesBase,
    TableSize,
    PtrOffset(DAVLattice),
    FnPtr,
    CheckedVal,
    CheckFlag(u32)
}

type CallCheckValueLattice = ConstLattice<CallCheckValue>;


// #[test]
// fn call_lattice_test() {
//     let x1  = SwitchValueLattice {v : None};
//     let x2  = SwitchValueLattice {v : Some(SwitchValue::SwitchBase(1))};
//     let x3  = SwitchValueLattice {v : Some(SwitchValue::SwitchBase(1))};
//     let x4  = SwitchValueLattice {v : Some(SwitchValue::SwitchBase(2))};
//     let x5  = SwitchValueLattice {v : Some(SwitchValue::UpperBound(1))};

//     assert_eq!(x1 == x2, false);
//     assert_eq!(x2 == x3, true);
//     assert_eq!(x3 == x4, false);
//     assert_eq!(x4 == x5, false);

//     assert_eq!(x1 != x2, true);
//     assert_eq!(x2 != x3, false);
//     assert_eq!(x3 != x4, true);
//     assert_eq!(x4 != x5, true);

//     assert_eq!(x1 > x2, false);
//     assert_eq!(x2 > x3, false);
//     assert_eq!(x3 > x4, false);
//     assert_eq!(x4 > x5, false);

//     assert_eq!(x1 < x2, true);
//     assert_eq!(x2 < x3, false);
//     assert_eq!(x3 < x4, false);
//     assert_eq!(x4 < x5, false);

//     assert_eq!(x1.meet(x2) == SwitchValueLattice {v : None}, true);
//     assert_eq!(x2.meet(x3) == SwitchValueLattice {v :  Some(SwitchValue::SwitchBase(1))}, true);
//     assert_eq!(x3.meet(x4) == SwitchValueLattice {v : None}, true);
//     assert_eq!(x4.meet(x5) == SwitchValueLattice {v : None}, true);


// }
