use std::cmp::Ordering;
use crate::lattices::{Lattice, Valued, Constu32Lattice};

enum HeapValue {
    SwitchBase(Constu32Lattice),
    UpperBound(Constu32Lattice),
    JmpOffset(Constu32Lattice),
    JmpTarget(ConstU32Lattice),
}

#[derive(Eq)]
pub struct HeapValueLattice{
    v: Option<u32>
}

// impl Valued for HeapValueLattice{
//     type Vtype = Option<HeapValue>;
//     fn value(&self) -> Self::Vtype {
//         self.v
//     }
}

impl PartialOrd for HeapValueLattice {
    fn partial_cmp(&self, other: &HeapValueLattice) -> Option<Ordering> {
        match (self.v, other.v){
            (None,None) => Some(Ordering::Equal),
            (None,_) => Some(Ordering::Less),
            (_,None) => Some(Ordering::Greater),
            (Some(x), Some(y)) => 
                if x == y {Some(Ordering::Equal) }
                else {None}
        }
    }
}

impl PartialEq for HeapValueLattice {
    fn eq(&self, other: &HeapValueLattice) -> bool {
        self.v == other.v
    }
}

impl Lattice for HeapValueLattice {
    fn meet(&self, other : Self) -> Self {
        if self.v == other.v {HeapValueLattice {v : self.v}}
        else {HeapValueLattice { v : None}}
    }
} 

impl Default for HeapValueLattice {
    fn default() -> Self {
        HeapValueLattice {v : None}
    }
}