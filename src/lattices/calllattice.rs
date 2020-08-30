use crate::lattices::VariableState;
use crate::lattices::Lattice;
use crate::lattices::davlattice::{DAV};
use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum CallCheckValue {
    GuestTableBase,
    LucetTablesBase,
    TableSize,
    PtrOffset(DAV),
    FnPtr,
    CheckedVal,
    CheckFlag(u32)
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct CallCheckValueLattice{
    pub v: Option<CallCheckValue>
}

pub type CallCheckLattice =  VariableState<CallCheckValueLattice>;


impl Default for CallCheckValueLattice {
    fn default() -> Self {
        CallCheckValueLattice {v : None}
    }
}

impl Lattice for CallCheckValueLattice {
    fn meet(&self, other : &Self) -> Self {
        if self.v == other.v {CallCheckValueLattice {v : self.v}}
        else {
            match (self.v, other.v){
                (Some(CallCheckValue::PtrOffset(x)),Some(CallCheckValue::PtrOffset(y))) => 
                    CallCheckValueLattice { v : Some(CallCheckValue::PtrOffset(x.meet(&y)))},
                (_,_) => CallCheckValueLattice { v : None}
            }
        }
    }
} 

impl PartialOrd for CallCheckValueLattice {
    fn partial_cmp(&self, other: &CallCheckValueLattice) -> Option<Ordering> {
        match (self.v, other.v){
            (None,None) => Some(Ordering::Equal),
            (None,_) => Some(Ordering::Less),
            (_,None) => Some(Ordering::Greater),
            (Some(x), Some(y)) =>{ 
                match (x, y){
                    (CallCheckValue::PtrOffset(x), CallCheckValue::PtrOffset(y) ) => x.partial_cmp(&y),
                    (_,_) =>  {
                        if x == y {Some(Ordering::Equal) }
                        else {None}}
                }
            } 
        }
    }
}


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
