use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::reachingdefslattice::ReachingDefnLattice;
use crate::lattices::Lattice;
use std::cmp::Ordering;

// Dependent Abstract Value
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DAV {
    Unknown,
    Unchecked(ReachingDefnLattice),
    Checked,
}

impl PartialOrd for DAV {
    fn partial_cmp(&self, other: &DAV) -> Option<Ordering> {
        match (self, other) {
            (DAV::Unknown, DAV::Unknown) => Some(Ordering::Equal),
            (DAV::Unknown, _) => Some(Ordering::Less),
            (DAV::Unchecked(x), DAV::Unchecked(y)) => {
                if x == y {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }
            (DAV::Unchecked(_), DAV::Checked) => Some(Ordering::Less),
            (DAV::Checked, DAV::Checked) => Some(Ordering::Equal),
            (DAV::Checked, DAV::Unchecked(y)) => Some(Ordering::Greater),
            (DAV::Unchecked(_), DAV::Unknown) => Some(Ordering::Greater),
            (DAV::Checked, DAV::Unknown) => Some(Ordering::Greater),
        }
    }
}

impl Lattice for DAV {
    fn meet(&self, other: &Self, _loc_idx: &LocIdx) -> Self {
        match (self, other) {
            (DAV::Unknown, _) => DAV::Unknown,
            (_, DAV::Unknown) => DAV::Unknown,
            (DAV::Unchecked(x), DAV::Unchecked(y)) => {
                if x == y {
                    self.clone()
                } else {
                    DAV::Unknown
                }
            }
            (DAV::Checked, DAV::Checked) => self.clone(),
            (DAV::Unchecked(_), DAV::Checked) => self.clone(),
            (DAV::Checked, DAV::Unchecked(_)) => other.clone(),
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
fn dav_lattice_test() {}
