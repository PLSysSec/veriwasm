use crate::lattices::davlattice::DAV;
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::{Lattice, VariableState};
use std::cmp::Ordering;

#[derive(Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum CallCheckValue {
    GuestTableBase,
    LucetTablesBase,
    TableSize,
    TypeOf(u8),//regnum
    PtrOffset(DAV),
    //TypedPtrOffset(u32),
    FnPtr,//type
    CheckedVal,
    CheckFlag(u32, u8),
    //TypeCheckFlag(u32, u8, u32),//addr, regnum, typeidx
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CallCheckValueLattice {
    pub v: Option<CallCheckValue>,
}

pub type CallCheckLattice = VariableState<CallCheckValueLattice>;

impl Default for CallCheckValueLattice {
    fn default() -> Self {
        CallCheckValueLattice { v: None }
    }
}

impl CallCheckValueLattice {
    pub fn new(v: CallCheckValue) -> Self {
        CallCheckValueLattice { v: Some(v) }
    }
}

impl Lattice for CallCheckValueLattice {
    fn meet(&self, other: &Self, loc_idx: &LocIdx) -> Self {
        if self.v.clone() == other.v {
            CallCheckValueLattice { v: self.v.clone() }
        } else {
            match (self.v.clone(), other.v.clone()) {
                (Some(CallCheckValue::PtrOffset(x)), Some(CallCheckValue::PtrOffset(y))) => {
                    CallCheckValueLattice {
                        v: Some(CallCheckValue::PtrOffset(x.meet(&y, loc_idx))),
                    }
                }
                (_, _) => CallCheckValueLattice { v: None },
            }
        }
    }
}

impl PartialOrd for CallCheckValueLattice {
    fn partial_cmp(&self, other: &CallCheckValueLattice) -> Option<Ordering> {
        match (self.v.clone(), other.v.clone()) {
            (None, None) => Some(Ordering::Equal),
            (None, _) => Some(Ordering::Less),
            (_, None) => Some(Ordering::Greater),
            (Some(x), Some(y)) => match (x.clone(), y.clone()) {
                (CallCheckValue::PtrOffset(x), CallCheckValue::PtrOffset(y)) => x.partial_cmp(&y),
                (_, _) => {
                    if x == y {
                        Some(Ordering::Equal)
                    } else {
                        None
                    }
                }
            },
        }
    }
}

#[test]
fn call_lattice_test() {
    let x1 = CallCheckValueLattice { v: None };
    let x2 = CallCheckValueLattice {
        v: Some(CallCheckValue::GuestTableBase),
    };
    let x3 = CallCheckValueLattice {
        v: Some(CallCheckValue::PtrOffset(DAV::Unknown)),
    };
    let x4 = CallCheckValueLattice {
        v: Some(CallCheckValue::PtrOffset(DAV::Unknown)),
    };
    let x5 = CallCheckValueLattice {
        v: Some(CallCheckValue::PtrOffset(DAV::Checked)),
    };

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

    assert_eq!(
        x1.meet(&x2, &LocIdx { addr: 0, idx: 0 }) == CallCheckValueLattice { v: None },
        true
    );
    assert_eq!(
        x2.meet(&x3, &LocIdx { addr: 0, idx: 0 }) == CallCheckValueLattice { v: None },
        true
    );
    assert_eq!(
        x3.meet(&x4, &LocIdx { addr: 0, idx: 0 })
            == CallCheckValueLattice {
                v: Some(CallCheckValue::PtrOffset(DAV::Unknown))
            },
        true
    );
    assert_eq!(
        x4.meet(&x5, &LocIdx { addr: 0, idx: 0 })
            == CallCheckValueLattice {
                v: Some(CallCheckValue::PtrOffset(DAV::Unknown))
            },
        true
    );
}
