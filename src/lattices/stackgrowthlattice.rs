use std::cmp::Ordering;
use crate::lattices::{Lattice, Valued Constu32Lattice};

#[derive(Eq)]
pub struct StackGrowthLattice{
    v: Option<u32>
}

// impl Valued for StackGrowthLattice{
//     type Vtype = Option<u32>;
//     fn value(&self) -> Self::Vtype {
//         self.v
//     }
// }

impl PartialOrd for StackGrowthLattice {
    fn partial_cmp(&self, other: &StackGrowthLattice) -> Option<Ordering> {
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

impl PartialEq for StackGrowthLattice {
    fn eq(&self, other: &StackGrowthLattice) -> bool {
        self.v == other.v
    }
}

impl Lattice for StackGrowthLattice {
    fn meet(&self, other : Self) -> Self {
        if self.v == other.v StackGrowthLattice {v : self.v}}
        else {StackGrowthLattice { v : None}}
    }
} 

impl Default for StackGrowthLattice {
    fn default() -> Self {
        StackGrowthLattice {v : 0}
    }
}