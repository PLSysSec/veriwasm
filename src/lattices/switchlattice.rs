use crate::ir::types::X86Regs;
use crate::lattices::reachingdefslattice::ReachingDefnLattice;
use crate::lattices::{ConstLattice, VariableState};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SwitchValue {
    SwitchBase(u32),
    ZF(u32, X86Regs, ReachingDefnLattice),
    UpperBound(u32),
    JmpOffset(u32, u32), // base + bound
    JmpTarget(u32, u32), //base + bound
}

pub type SwitchValueLattice = ConstLattice<SwitchValue>;

pub type SwitchLattice = VariableState<SwitchValueLattice>;

#[test]
fn switch_lattice_test() {
    use crate::lattices::reachingdefslattice::LocIdx;
    use crate::lattices::Lattice;

    let x1 = SwitchValueLattice { v: None };
    let x2 = SwitchValueLattice {
        v: Some(SwitchValue::SwitchBase(1)),
    };
    let x3 = SwitchValueLattice {
        v: Some(SwitchValue::SwitchBase(1)),
    };
    let x4 = SwitchValueLattice {
        v: Some(SwitchValue::SwitchBase(2)),
    };
    let x5 = SwitchValueLattice {
        v: Some(SwitchValue::UpperBound(1)),
    };

    assert_eq!(x1 == x2, false);
    assert_eq!(x2 == x3, true);
    assert_eq!(x3 == x4, false);
    assert_eq!(x4 == x5, false);

    assert_eq!(x1 != x2, true);
    assert_eq!(x2 != x3, false);
    assert_eq!(x3 != x4, true);
    assert_eq!(x4 != x5, true);

    assert_eq!(x1 > x2, false);
    assert_eq!(x2 > x3, false);
    assert_eq!(x3 > x4, false);
    assert_eq!(x4 > x5, false);

    assert_eq!(x1 < x2, true);
    assert_eq!(x2 < x3, false);
    assert_eq!(x3 < x4, false);
    assert_eq!(x4 < x5, false);

    assert_eq!(
        x1.meet(&x2, &LocIdx { addr: 0, idx: 0 }) == SwitchValueLattice { v: None },
        true
    );
    assert_eq!(
        x2.meet(&x3, &LocIdx { addr: 0, idx: 0 })
            == SwitchValueLattice {
                v: Some(SwitchValue::SwitchBase(1))
            },
        true
    );
    assert_eq!(
        x3.meet(&x4, &LocIdx { addr: 0, idx: 0 }) == SwitchValueLattice { v: None },
        true
    );
    assert_eq!(
        x4.meet(&x5, &LocIdx { addr: 0, idx: 0 }) == SwitchValueLattice { v: None },
        true
    );
}
