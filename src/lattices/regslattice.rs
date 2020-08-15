// pub mod lattices 
use std::cmp::Ordering;
use crate::lattices::{Lattice, BooleanLattice};

//TODO: fix constructor so we only need to pass some arguments?
#[derive(Eq)]
#[derive(Default)]
pub struct X86RegsLattice<T:Lattice>{
    rax : T,
    rbx : T,
    rcx : T,
    rdx : T,
    rdi  : T,
    rsi  : T,
    rsp  : T,
    rbp  : T,
    r8  : T,
    r9  : T,
    r10  : T,
    r11  : T,
    r12  : T,
    r13  : T,
    r14  : T,
    r15  : T,
    zf : T
}

//TODO: fix by implementing iterator on regslattice and using zip + all
impl<T:Lattice> PartialOrd for X86RegsLattice<T> {
    fn partial_cmp(&self, other: &X86RegsLattice<T>) -> Option<Ordering> {
        if 
        self.rax > other.rax && self.rbx > other.rbx && self.rcx > other.rcx && self.rdx > other.rdx &&
        self.rdi > other.rdi && self.rsi > other.rsi && self.rsp > other.rsp && self.rbp > other.rbp && self.r8 > other.r8 &&
        self.r9 > other.r9 && self.r10 > other.r10 && self.r11 > other.r11 && self.r12 > other.r12 && 
        self.r13 > other.r13 && self.r14 > other.r14 && self.r15 > other.r15 && self.zf > other.zf  
        {Some(Ordering::Greater)}
        else if 
        self.rax < other.rax && self.rbx < other.rbx && self.rcx < other.rcx && self.rdx < other.rdx &&
        self.rdi < other.rdi && self.rsi < other.rsi && self.rsp < other.rsp && self.rbp < other.rbp && self.r8 < other.r8 &&
        self.r9 < other.r9 && self.r10 < other.r10 && self.r11 < other.r11 && self.r12 < other.r12 && 
        self.r13 < other.r13 && self.r14 < other.r14 && self.r15 < other.r15 && self.zf < other.zf 
        {Some(Ordering::Less)}
        else if 
        self.rax == other.rax && self.rbx == other.rbx && self.rcx == other.rcx && self.rdx == other.rdx &&
        self.rdi == other.rdi && self.rsi == other.rsi && self.rsp == other.rsp && self.rbp == other.rbp && self.r8 == other.r8 &&
        self.r9 == other.r9 && self.r10 == other.r10 && self.r11 == other.r11 && self.r12 == other.r12 && 
        self.r13 == other.r13 && self.r14 == other.r14 && self.r15 == other.r15 && self.zf == other.zf 
        {Some(Ordering::Equal)}
        else {None}
    }
}

impl<T:Lattice> PartialEq for X86RegsLattice<T> {
    fn eq(&self, other: &X86RegsLattice<T>) -> bool {
        self.rax == other.rax && self.rbx == other.rbx && self.rcx == other.rcx && self.rdx == other.rdx &&
        self.rdi == other.rdi && self.rsi == other.rsi && self.rsp == other.rsp && self.rbp == other.rbp && self.r8 == other.r8 &&
        self.r9 == other.r9 && self.r10 == other.r10 && self.r11 == other.r11 && self.r12 == other.r12 && 
        self.r13 == other.r13 && self.r14 == other.r14 && self.r15 == other.r15 && self.zf == other.zf 
    }
}

impl<T:Lattice> Lattice for X86RegsLattice<T> {
    fn meet(&self, other : Self) -> Self {
        X86RegsLattice {
            rax : self.rax.meet(other.rax), 
            rbx: self.rbx.meet(other.rbx), 
            rcx : self.rcx.meet(other.rcx),
            rdx : self.rdx.meet(other.rdx),
            rdi : self.rdi.meet(other.rdi),
            rsi : self.rsi.meet(other.rsi),
            rsp : self.rsp.meet(other.rsp),
            rbp : self.rbp.meet(other.rbp),
            r8 : self.r8.meet(other.r8),
            r9 : self.r9.meet(other.r9),
            r10 : self.r10.meet(other.r10),
            r11 : self.r11.meet(other.r11),
            r12 : self.r12.meet(other.r12),
            r13 : self.r13.meet(other.r13),
            r14 : self.r14.meet(other.r14),
            r15 : self.r15.meet(other.r15),
            zf : self.zf.meet(other.zf)
        }
    }
} 


#[test]
fn regs_lattice_test() {
    let x  = BooleanLattice {v : false};
    let y  = BooleanLattice {v : true};

    let r = X86RegsLattice {
        rax : x, 
        rbx: y, 
        rcx : BooleanLattice {v : false},
        rdx : BooleanLattice {v : false},
        rdi : BooleanLattice {v : false},
        rsi : BooleanLattice {v : false},
        rsp : BooleanLattice {v : false},
        rbp : BooleanLattice {v : false},
        r8 : BooleanLattice {v : false},
        r9 : BooleanLattice {v : false},
        r10 : BooleanLattice {v : false},
        r11 : BooleanLattice {v : false},
        r12 : BooleanLattice {v : false},
        r13 : BooleanLattice {v : false},
        r14 : BooleanLattice {v : false},
        r15 : BooleanLattice {v : false},
        zf : BooleanLattice {v : false}
    };
    assert_eq!(r.rax < r.rbx, true);
    assert_eq!(r.rax > r.rbx, false);
    assert_eq!(r.rax.lt(&r.rbx), true);
    assert_eq!(r.rax == r.rdi, true);
}
