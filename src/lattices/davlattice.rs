use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::reachingdefslattice::ReachingDefnLattice;
use std::cmp::Ordering;
use crate::lattices::{Lattice};

// Dependent Abstract Value
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DAV {
    Unknown,
    Unchecked(ReachingDefnLattice),
    Checked,
}

// #[derive(Eq, Clone, Copy)]
// pub struct DAVLattice{
//     v: DAV
// }

impl PartialOrd for DAV {
    fn partial_cmp(&self, other: &DAV) -> Option<Ordering> {
        match (self, other){
            (DAV::Unknown,DAV::Unknown) => Some(Ordering::Equal),
            (DAV::Unknown,_) => Some(Ordering::Less),
            (DAV::Unchecked(x),DAV::Unchecked(y)) =>  
                if x == y {Some(Ordering::Equal) }
                else {None},
            (DAV::Unchecked(_),DAV::Checked) => Some(Ordering::Less),
            (DAV::Checked,DAV::Checked) => Some(Ordering::Equal),
            (DAV::Checked,DAV::Unchecked(y)) => Some(Ordering::Greater),
            (DAV::Unchecked(_),DAV::Unknown) => Some(Ordering::Greater),
            (DAV::Checked,DAV::Unknown) => Some(Ordering::Greater),
        }
    }
}

// impl PartialEq for DAV {
//     fn eq(&self, other: &DAV) -> bool {
//         self.v == other.v
//     }
// }

impl Lattice for DAV {
    fn meet(&self, other : &Self, loc_idx : &LocIdx) -> Self {
        match (self, other){
            (DAV::Unknown,_) => DAV::Unknown,
            (_,DAV::Unknown) => DAV::Unknown,
            (DAV::Unchecked(x),DAV::Unchecked(y)) =>  
                if x == y {self.clone()}
                else {DAV::Unknown},
            (DAV::Checked,DAV::Checked) => self.clone(),
            (DAV::Unchecked(_),DAV::Checked) => self.clone(),
            (DAV::Checked,DAV::Unchecked(_)) => other.clone()
        }
    }
} 

//What is a default index? This might be wrong.
//Perhaps default is what index should be at start of function.
impl Default for DAV {
    fn default() -> Self {
         DAV::Unknown
    }
}

#[test]
fn dav_lattice_test() {
    // let x1 = DAV::Unknown;
    // let x2 = DAV::Unchecked(1);
    // let x3 = DAV::Unchecked(1);
    // let x4 = DAV::Unchecked(2);
    // let x5 = DAV::Checked;

    // assert_eq!(x1 == x2, false);
    // assert_eq!(x2 == x3, true);
    // assert_eq!(x3 == x4, false);
    // assert_eq!(x4 == x5, false);

    // assert_eq!(x1 != x2, true);
    // assert_eq!(x2 != x3, false);
    // assert_eq!(x3 != x4, true);
    // assert_eq!(x4 != x5, true);

    // assert_eq!(x1 > x2, false);
    // assert_eq!(x2 > x3, false);
    // assert_eq!(x3 > x4, false);
    // assert_eq!(x4 > x5, false);

    // assert_eq!(x1 < x2, true);
    // assert_eq!(x2 < x3, false);
    // assert_eq!(x3 < x4, false);
    // assert_eq!(x4 < x5, true);

    // assert_eq!(x1.meet(&x2) == DAV::Unknown, true);
    // assert_eq!(x2.meet(&x3) == DAV::Unchecked(1), true);
    // assert_eq!(x3.meet(&x4) == DAV::Unknown, true);
    // assert_eq!(x4.meet(&x5) == DAV::Unchecked(2), true);
}

