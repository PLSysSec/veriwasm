use crate::ir::types::{Binopcode, Value};
use crate::lattices::{ConstLattice, VarState};

/// Fields are: (stackgrowth, probestack, rbp_stackgrowth)
///
/// `stackgrowth` is a value that indicates how far downward the stack is valid.
///
/// `rbp_stackgrowth` is a copy of `stackgrowth` that is saved when `rsp` is
/// copied to `rbp` during frame setup in the prologue. When `rsp` is restored
/// from `rbp` in the epilogue, it is copied back.
pub type StackGrowthLattice = ConstLattice<(i64, i64, i64)>;

impl VarState for StackGrowthLattice {
    type Var = i64;
    fn get(&self, _index: &Value) -> Option<Self::Var> {
        unimplemented!()
    }
    fn set(&mut self, _index: &Value, _v: Self::Var) -> () {
        unimplemented!()
    }
    fn set_to_bot(&mut self, _index: &Value) -> () {
        unimplemented!()
    }
    fn on_call(&mut self) -> () {
        unimplemented!()
    }
    fn adjust_stack_offset(
        &mut self,
        _opcode: &Binopcode,
        _dst: &Value,
        _src1: &Value,
        _src2: &Value,
    ) {
        unimplemented!()
    }
}

impl StackGrowthLattice {
    pub fn get_stackgrowth(&self) -> Option<i64> {
        match self.v {
            Some((stackgrowth, _, _)) => Some(stackgrowth),
            None => None,
        }
    }

    pub fn get_probestack(&self) -> Option<i64> {
        match self.v {
            Some((_, probestack, _)) => Some(probestack),
            None => None,
        }
    }

    pub fn get_rbp(&self) -> Option<i64> {
        match self.v {
            Some((_, _, rbp)) => Some(rbp),
            None => None,
        }
    }
}

#[test]
fn stack_growth_lattice_test() {
    use crate::ir::types::LocIdx;
    use crate::lattices::Lattice;

    let x1 = StackGrowthLattice { v: None };
    let x2 = StackGrowthLattice {
        v: Some((1, 4096, 0)),
    };
    let x3 = StackGrowthLattice {
        v: Some((1, 4096, 0)),
    };
    let x4 = StackGrowthLattice {
        v: Some((2, 4096, 0)),
    };

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

    assert_eq!(
        x1.meet(&x2, &LocIdx { addr: 0, idx: 0 }) == StackGrowthLattice { v: None },
        true
    );
    assert_eq!(
        x2.meet(&x3, &LocIdx { addr: 0, idx: 0 })
            == StackGrowthLattice {
                v: Some((1, 4096, 0))
            },
        true
    );
    assert_eq!(
        x3.meet(&x4, &LocIdx { addr: 0, idx: 0 }) == StackGrowthLattice { v: None },
        true
    );
}
