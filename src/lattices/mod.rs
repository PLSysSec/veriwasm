use std::cmp::Ordering;
pub mod regslattice;


// pub trait Valued {
//     type Vtype;        
//     // fn value(&self) -> Self::Vtype;

//     fn value(&self) -> Self::Vtype {
//         unimplemented!();
//     }
// }


// pub trait Meetable {
//     fn meet(&self, other : Self) -> Self;
// }

pub trait Lattice: PartialOrd + PartialEq + Default {
    fn meet(&self, other : Self) -> Self;
}
// impl<T> Lattice for T where T: PartialOrd + PartialEq+ Valued + Default {}

#[derive(Eq)]
pub struct BooleanLattice{
    v: bool
}

// impl Valued for BooleanLattice{
//     type Vtype = bool;
//     fn value(&self) -> Self::Vtype {
//         self.v
//     }
// }

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


#[derive(Eq)]
pub struct Constu32Lattice{
    v: Option<u32>
}

// impl Valued for Constu32Lattice{
//     type Vtype = Option<u32>;
//     fn value(&self) -> Self::Vtype {
//         self.v
//     }
// }

impl PartialOrd for Constu32Lattice {
    fn partial_cmp(&self, other: &Constu32Lattice) -> Option<Ordering> {
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

impl PartialEq for Constu32Lattice {
    fn eq(&self, other: &Constu32Lattice) -> bool {
        self.v == other.v
    }
}

impl Lattice for Constu32Lattice {
    fn meet(&self, other : Self) -> Self {
        if self.v == other.v {Constu32Lattice {v : self.v}}
        else {Constu32Lattice { v : None}}
    }
} 

impl Default for Constu32Lattice {
    fn default() -> Self {
        Constu32Lattice {v : None}
    }
}

// #[derive(Eq)]
// pub struct CompositeLattice<T:Lattice>{
//     v: Vec<T>
// }

// impl<T:Lattice> Valued for CompositeLattice<T>{
//     type Vtype = Vec<T>;
//     fn value(&self) -> Self::Vtype {
//         self.v.clone()
//     }
// }

// impl<T:Lattice> PartialOrd for CompositeLattice<T> {
//     fn partial_cmp(&self, other: &CompositeLattice<T>) -> Option<Ordering> {
//         assert_eq!(self.v.len() == other.v.len(), true);
//         self.v.partial_cmp(&other.v)
//     }
// }

// impl<T:Lattice> PartialEq for CompositeLattice<T>{
//     fn eq(&self, other: &CompositeLattice<T>) -> bool {
//         self.v == other.v
//     }
// }



pub fn hello_world(){
    println!("Hello, world!");
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
    let x1  = Constu32Lattice {v : Some(1)};
    let x2  = Constu32Lattice {v : Some(1)};

    let y1  = Constu32Lattice {v : Some(2)};
    let y2  = Constu32Lattice {v : Some(2)};
    assert_eq!(x1 < y1, false);
    assert_eq!(y1 < x1, false);
    assert_eq!(x1 == x2, true);
    assert_eq!(x1 != x2, false);
    assert_eq!(y2 != x1, true);
    assert_eq!(x1 >= y1, false);
    assert_eq!(x1 > x2, false);
    assert_eq!(x1 >= x2, true);
    assert_eq!(x1.lt(&y1), false);
}


// #[test]
// fn composite_lattice_test() {
//     let f1  = BooleanLattice {v : false};
//     let f2  = BooleanLattice {v : false};
//     let t1  = BooleanLattice {v : true};
//     let t2  = BooleanLattice {v : true};
//     let f3  = BooleanLattice {v : false};
//     let f4  = BooleanLattice {v : false};
//     let t3  = BooleanLattice {v : true};
//     let t4  = BooleanLattice {v : true};

//     let mut vecft = vec![f1, t1];
//     let mut vectf = vec![t2, f2];
//     let mut vecff = vec![f3, f4];
//     let mut vectt = vec![t3, t4];


//     assert_eq!(vecft < vectf, false);
//     assert_eq!(vecff < vectt, true);
//     assert_eq!(vecff < vecft, true);
//     assert_eq!(vecff == vectf, false);

// }
