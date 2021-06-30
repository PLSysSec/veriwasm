use std::convert::TryFrom;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::ir::types::{ValSize ,X86Regs};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::{Lattice, VarSlot};

use X86Regs::*;

#[derive(Default, PartialEq, Eq, Clone, Debug)]
pub struct X86RegsLattice<T> {
    pub map: HashMap<X86Regs, VarSlot<T>>
}

fn hashmap_le<T: PartialOrd>(s1: &X86RegsLattice<T>, s2: &X86RegsLattice<T>) -> bool {
    for (k1, v1) in s1.map.iter() {
        if !s2.map.contains_key(k1) {
            return false;
        } else {
            if s2.map.get(k1).unwrap() < v1 {
                return false;
            } else {
            }
        }
    }
    true
}

impl<T: PartialOrd> PartialOrd for X86RegsLattice<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if hashmap_le(self, other) {
            Some(Ordering::Less)
        } else if hashmap_le(other, self) {
            Some(Ordering::Greater)
        } else if self == other {
            Some(Ordering::Equal)
        } else {
            None
        }
    }
}

impl<T: Lattice + Clone> X86RegsLattice<T> {
    pub fn get_reg(&self, index: X86Regs, size: ValSize) -> T {
        if let ValSize::SizeOther = size {
            return Default::default(); // TODO: what is happening here
        }
        if let Some(slot) = self.map.get(&index) {
            slot.value.clone()
        } else {
            Default::default()
        }
    }

    pub fn get_reg_index(&self, index: u8, size: ValSize) -> T {
        let reg_index = match X86Regs::try_from(index) {
            Err(err) => panic!("{}", err),
            Ok(reg) => reg,
        };
        self.get_reg(reg_index, size)
    }

    pub fn set_reg(&mut self, index: X86Regs, size: ValSize, value: T) {
        if let ValSize::SizeOther = size {
            return; // TODO: what is happening here
        }
        self.map.insert(index, VarSlot { size: size.into_bits(), value });
    }

    pub fn set_reg_index(&mut self, index: u8, size: ValSize, value: T) -> () {
        let reg_index = match X86Regs::try_from(index) {
            Err(err) => panic!("{}", err),
            Ok(reg) => reg,
        };
        self.set_reg(reg_index, size, value)
    }

    pub fn clear_regs(&mut self) -> () {
        self.map.clear()
    }

    // TODO: should this do the inverse?
    pub fn clear_caller_save_regs(&mut self) {
        // x86-64 calling convention: rax, rcx, rdx, rsi, rdi, r8, r9, r10, r11 must be saved by
        // the caller (are clobbered by the callee), so their states become unknown after calls.
        //
        // TODO: get calling convention from program's target ABI; on Windows, rsi and rdi are
        // callee-save. The below is thus sound but conservative (and possibly
        // false-positive-producing) on Windows.
        self.map.remove(&Rax);
        self.map.remove(&Rcx);
        self.map.remove(&Rdx);
        self.map.remove(&Rsi);
        self.map.remove(&Rdi);

        self.map.remove(&R8);
        self.map.remove(&R9);
        self.map.remove(&R10);
        self.map.remove(&R11);
        self.map.remove(&Zf);
        self.map.remove(&Cf);
    }

    pub fn show(&self) -> () {
        println!("State = ");
        println!("{:?}", self.map);
    }
}

impl<T: Lattice + Clone> Lattice for X86RegsLattice<T> {
    fn meet(&self, other: &Self, loc_idx: &LocIdx) -> Self {
        let mut newmap: HashMap<X86Regs, VarSlot<T>> = HashMap::new();
        for (var_index, v1) in self.map.iter() {
            match other.map.get(var_index) {
                Some(v2) => {
                    // TODO(matt): what if the sizes are different?
                    if v1.size == v2.size {
                        let new_v = v1.value.meet(&v2.value.clone(), loc_idx);
                        let newslot = VarSlot {
                            size: v1.size,
                            value: new_v,
                        };
                        newmap.insert(*var_index, newslot);
                    }
                }
                None => () // this means v2 = ⊥ so v1 ∧ v2 = ⊥
            }
        }
        X86RegsLattice { map: newmap }
    }
}

// TODO: put this back
// #[test]
// fn regs_lattice_test() {
//     use crate::lattices::BooleanLattice;

//     let r1 = X86RegsLattice {
//         rax: BooleanLattice { v: false },
//         rbx: BooleanLattice { v: false },
//         rcx: BooleanLattice { v: false },
//         rdx: BooleanLattice { v: false },
//         rdi: BooleanLattice { v: false },
//         rsi: BooleanLattice { v: false },
//         rsp: BooleanLattice { v: false },
//         rbp: BooleanLattice { v: false },
//         r8: BooleanLattice { v: false },
//         r9: BooleanLattice { v: false },
//         r10: BooleanLattice { v: false },
//         r11: BooleanLattice { v: false },
//         r12: BooleanLattice { v: false },
//         r13: BooleanLattice { v: false },
//         r14: BooleanLattice { v: false },
//         r15: BooleanLattice { v: false },
//         zf: BooleanLattice { v: false },
//     };

//     let r2 = X86RegsLattice {
//         rax: BooleanLattice { v: true },
//         rbx: BooleanLattice { v: false },
//         rcx: BooleanLattice { v: false },
//         rdx: BooleanLattice { v: false },
//         rdi: BooleanLattice { v: false },
//         rsi: BooleanLattice { v: false },
//         rsp: BooleanLattice { v: false },
//         rbp: BooleanLattice { v: false },
//         r8: BooleanLattice { v: false },
//         r9: BooleanLattice { v: false },
//         r10: BooleanLattice { v: false },
//         r11: BooleanLattice { v: false },
//         r12: BooleanLattice { v: false },
//         r13: BooleanLattice { v: false },
//         r14: BooleanLattice { v: false },
//         r15: BooleanLattice { v: false },
//         zf: BooleanLattice { v: false },
//     };

//     let r3 = X86RegsLattice {
//         rax: BooleanLattice { v: false },
//         rbx: BooleanLattice { v: true },
//         rcx: BooleanLattice { v: false },
//         rdx: BooleanLattice { v: false },
//         rdi: BooleanLattice { v: false },
//         rsi: BooleanLattice { v: false },
//         rsp: BooleanLattice { v: false },
//         rbp: BooleanLattice { v: false },
//         r8: BooleanLattice { v: false },
//         r9: BooleanLattice { v: false },
//         r10: BooleanLattice { v: false },
//         r11: BooleanLattice { v: false },
//         r12: BooleanLattice { v: false },
//         r13: BooleanLattice { v: false },
//         r14: BooleanLattice { v: false },
//         r15: BooleanLattice { v: false },
//         zf: BooleanLattice { v: false },
//     };

//     assert_eq!(r2.rax > r2.rbx, true);
//     assert_eq!(r2.rax < r2.rbx, false);
//     assert_eq!(r2.rax.gt(&r2.rbx), true);
//     assert_eq!(r2.rbx == r2.rdi, true);

//     assert_eq!(r1 < r2, true);
//     assert_eq!(r1 <= r2, true);

//     assert_eq!(r2 < r3, false);
//     assert_eq!(r2 <= r3, false);

//     assert_eq!(r2.meet(&r3, &LocIdx { addr: 0, idx: 0 }) == r1, true);
//     assert_eq!(r1.meet(&r2, &LocIdx { addr: 0, idx: 0 }) == r1, true);
// }
