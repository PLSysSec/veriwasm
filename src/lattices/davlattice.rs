use std::cmp::Ordering;
use crate::lattices::{Lattice};

// Dependent Abstract Value
#[derive(Clone, Copy, PartialEq, Eq)]
enum DAV {
    Unknown,
    Unchecked(u64),
    Checked,
}

#[derive(Eq, Clone, Copy)]
pub struct DAVLattice{
    v: DAV
}

impl PartialOrd for DAVLattice {
    fn partial_cmp(&self, other: &DAVLattice) -> Option<Ordering> {
        match (self.v, other.v){
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

impl PartialEq for DAVLattice {
    fn eq(&self, other: &DAVLattice) -> bool {
        self.v == other.v
    }
}

impl Lattice for DAVLattice {
    fn meet(&self, other : Self) -> Self {
        match (self.v, other.v){
            (DAV::Unknown,_) => DAVLattice {v : DAV::Unknown},
            (_,DAV::Unknown) => DAVLattice {v : DAV::Unknown},
            (DAV::Unchecked(x),DAV::Unchecked(y)) =>  
                if x == y { DAVLattice {v : self.v} }
                else {DAVLattice {v : DAV::Unknown}},
            (DAV::Checked,DAV::Checked) => DAVLattice {v : self.v},
            (DAV::Unchecked(_),DAV::Checked) => { DAVLattice {v : self.v} },
            (DAV::Checked,DAV::Unchecked(_)) => { DAVLattice {v : other.v} }
        }
    }
} 

//What is a default index? This might be wrong.
//Perhaps default is what index should be at start of function.
impl Default for DAVLattice {
    fn default() -> Self {
        DAVLattice {v : DAV::Unknown}
    }
}

fn dav_lattice_test() {
    let x1 = DAVLattice {v : DAV::Unknown};
    let x2 = DAVLattice {v : DAV::Unchecked(1)};
    let x3 = DAVLattice {v : DAV::Unchecked(1)};
    let x4 = DAVLattice {v : DAV::Unchecked(2)};
    let x5 = DAVLattice {v : DAV::Checked};

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
    assert_eq!(x4 < x5, true);

    assert_eq!(x1.meet(x2) == DAVLattice {v : DAV::Unknown}, true);
    assert_eq!(x2.meet(x3) == DAVLattice {v : DAV::Unchecked(1)}, true);
    assert_eq!(x3.meet(x4) == DAVLattice {v : DAV::Unknown}, true);
    assert_eq!(x4.meet(x5) == DAVLattice {v : DAV::Unchecked(2)}, true);
}

