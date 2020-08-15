use std::cmp::Ordering;
pub mod regslattice;
pub mod heaplattice;
pub mod switchlattice;
pub mod davlattice;

pub trait Lattice: PartialOrd + Eq + Default {
    fn meet(&self, other : Self) -> Self;
}

#[derive(Eq)]
pub struct BooleanLattice{
    v: bool
}

impl PartialOrd for BooleanLattice {
    fn partial_cmp(&self, other: &BooleanLattice) -> Option<Ordering> {
        Some(self.v.cmp(&other.v))
    }
}

impl PartialEq for BooleanLattice {
    fn eq(&self, other: &BooleanLattice) -> bool {
        self.v == other.v
    }
}

impl Lattice for BooleanLattice {
    fn meet(&self, other : Self) -> Self {
        BooleanLattice {v : self.v && other.v}
    }
} 

impl Default for BooleanLattice {
    fn default() -> Self {
        BooleanLattice {v : false}
    }
}

type Constu32Lattice = ConstLattice::<u32>;

#[derive(Eq, Clone, Copy)]
pub struct ConstLattice<T:Eq + Copy>{
    v: Option<T>
}

impl<T:Eq + Copy> PartialOrd for ConstLattice<T> {
    fn partial_cmp(&self, other: &ConstLattice<T>) -> Option<Ordering> {
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

impl<T:Eq + Copy> PartialEq for ConstLattice<T> {
    fn eq(&self, other: &ConstLattice<T>) -> bool {
        self.v == other.v
    }
}

impl<T:Eq + Copy> Lattice for ConstLattice<T> {
    fn meet(&self, other : Self) -> Self {
        if self.v == other.v {ConstLattice {v : self.v}}
        else {ConstLattice { v : None}}
    }
} 

impl<T:Eq + Copy> Default for ConstLattice<T> {
    fn default() -> Self {
        ConstLattice {v : None}
    }
}



#[test]
fn boolean_lattice_test() {
    let x  = BooleanLattice {v : false};
    let y  = BooleanLattice {v : true};
    assert_eq!(x < y, true);
    assert_eq!(x > y, false);
    assert_eq!(x.lt(&y), true);
}

#[test]
fn u32_lattice_test() {
    let x1  = ConstLattice::<u32> {v : Some(1)};
    let x2  = ConstLattice::<u32> {v : Some(1)};
    let y1  = ConstLattice::<u32> {v : Some(2)};
    let y2  = ConstLattice::<u32> {v : Some(2)};

    let z1  = Constu32Lattice {v : Some(3)};
    let z2  = Constu32Lattice {v : Some(3)};

    // let y1  = Constu32Lattice {v : Some(2)};
    // let y2  = Constu32Lattice {v : Some(2)};
    assert_eq!(x1 < y1, false);
    assert_eq!(y1 < x1, false);
    assert_eq!(x1 == x2, true);
    assert_eq!(x1 != x2, false);
    assert_eq!(y2 != x1, true);
    assert_eq!(x1 >= y1, false);
    assert_eq!(x1 > x2, false);
    assert_eq!(x1 >= x2, true);
    assert_eq!(z1 == z2, true);
    assert_eq!(z1 == x1, false);
    assert_eq!(x1.lt(&y1), false);
}
