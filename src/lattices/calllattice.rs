use crate::lattices::{ConstLattice, Lattice};
use crate::lattices::davlattice::{DAVLattice, DAV};

//TODO: I think this does not treat the PtrOffset correctly

#[derive(Clone, Copy, PartialEq, Eq)]
enum CallCheckValue {
    GuestTableBase,
    LucetTablesBase,
    TableSize,
    PtrOffset(DAV),
    FnPtr,
    CheckedVal,
    CheckFlag(u32)
}

pub type CallCheckValueLattice = ConstLattice<CallCheckValue>;


#[test]
fn call_lattice_test() {
    let x1  = CallCheckValueLattice {v : None};
    let x2  = CallCheckValueLattice {v : Some(CallCheckValue::GuestTableBase)};
    let x3  = CallCheckValueLattice {v : Some(CallCheckValue::PtrOffset(DAV::Unknown))};
    let x4  = CallCheckValueLattice {v : Some(CallCheckValue::PtrOffset(DAV::Unknown))};
    let x5  = CallCheckValueLattice {v : Some(CallCheckValue::PtrOffset(DAV::Checked))};

    assert_eq!(x1 == x2, false);
    assert_eq!(x2 == x3, false);
    assert_eq!(x3 == x4, true);
    assert_eq!(x4 == x5, false);

    assert_eq!(x1 != x2, true);
    assert_eq!(x2 != x3, true);
    assert_eq!(x3 != x4, false);
    assert_eq!(x4 != x5, true);

    assert_eq!(x1 > x2, false);
    assert_eq!(x2 > x3, false);
    assert_eq!(x3 > x4, false);
    assert_eq!(x4 > x5, false);

    assert_eq!(x1 < x2, true);
    assert_eq!(x2 < x3, false);
    assert_eq!(x3 < x4, false);
    assert_eq!(x4 < x5, true);

    assert_eq!(x1.meet(&x2) == CallCheckValueLattice {v : None}, true);
    assert_eq!(x2.meet(&x3) == CallCheckValueLattice {v : None}, true);
    assert_eq!(x3.meet(&x4) == CallCheckValueLattice {v : Some(CallCheckValue::PtrOffset(DAV::Unknown))}, true);
    assert_eq!(x4.meet(&x5) ==CallCheckValueLattice {v : Some(CallCheckValue::PtrOffset(DAV::Unknown))}, true);


}
