use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::ir::types::{ValSize, X86Regs, RegT};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::{Lattice, VarSlot};
use std::hash::Hash;

// use X86Regs::*;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ArchRegsLattice<Ar: RegT, T> {
    pub map: HashMap<Ar, VarSlot<T>>,
}

fn hashmap_le<Ar: RegT, T: PartialOrd>(s1: &ArchRegsLattice<Ar, T>, s2: &ArchRegsLattice<Ar, T>) -> bool {
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

impl<Ar: RegT, T: PartialOrd> PartialOrd for ArchRegsLattice<Ar, T> {
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

impl<Ar: RegT, T: Lattice + Clone> ArchRegsLattice<Ar, T> {
    pub fn get_reg(&self, index: Ar, size: ValSize) -> T {
        if let Some(slot) = self.map.get(&index) {
            slot.value.clone()
        } else {
            Default::default()
        }
    }

    pub fn get_reg_index(&self, index: u8, size: ValSize) -> T {
        let reg_index = match Ar::try_from(index) {
            Err(err) => panic!("{}", err),
            Ok(reg) => reg,
        };
        self.get_reg(reg_index, size)
    }

    pub fn set_reg(&mut self, index: Ar, size: ValSize, value: T) {
        self.map.insert(
            index,
            VarSlot {
                size: size.into_bits(),
                value,
            },
        );
    }

    pub fn set_reg_index(&mut self, index: u8, size: ValSize, value: T) -> () {
        let reg_index = match Ar::try_from(index) {
            Err(err) => panic!("{}", err),
            Ok(reg) => reg,
        };
        self.set_reg(reg_index, size, value)
    }

    pub fn clear_regs(&mut self) -> () {
        self.map.clear()
    }

    // TODO: should this do the inverse?
    // pub fn clear_caller_save_regs(&mut self) {
    //     // x86-64 calling convention: rax, rcx, rdx, rsi, rdi, r8, r9, r10, r11 must be saved by
    //     // the caller (are clobbered by the callee), so their states become unknown after calls.
    //     //
    //     // TODO: get calling convention from program's target ABI; on Windows, rsi and rdi are
    //     // callee-save. The below is thus sound but conservative (and possibly
    //     // false-positive-producing) on Windows.
    //     self.map.remove(&X86Regs::Rax);
    //     self.map.remove(&X86Regs::Rcx);
    //     self.map.remove(&X86Regs::Rdx);
    //     self.map.remove(&X86Regs::Rsi);
    //     self.map.remove(&X86Regs::Rdi);

    //     self.map.remove(&X86Regs::R8);
    //     self.map.remove(&X86Regs::R9);
    //     self.map.remove(&X86Regs::R10);
    //     self.map.remove(&X86Regs::R11);
    //     self.map.remove(&X86Regs::Zf);
    //     self.map.remove(&X86Regs::Cf);
    //     self.map.remove(&X86Regs::Pf);
    //     self.map.remove(&X86Regs::Sf);
    //     self.map.remove(&X86Regs::Of);
    // }

    pub fn show(&self) -> () {
        println!("State = ");
        println!("{:?}", self.map);
    }
}

// Don't derive default because it requires regs to have a default as well
// https://github.com/rust-lang/rust/issues/26925
impl<Ar: RegT, T> Default for ArchRegsLattice<Ar, T> {
    fn default() -> Self {
        ArchRegsLattice {
            map: Default::default(),
        }
    }
}

impl<Ar: RegT, T: Lattice + Clone> Lattice for ArchRegsLattice<Ar, T> {
    fn meet(&self, other: &Self, loc_idx: &LocIdx) -> Self {
        let mut newmap: HashMap<Ar, VarSlot<T>> = HashMap::new();
        for (var_index, v1) in self.map.iter() {
            match other.map.get(var_index) {
                Some(v2) => {
                    let new_v = v1.value.meet(&v2.value.clone(), loc_idx);
                    let newslot = VarSlot {
                        size: std::cmp::min(v1.size, v2.size),
                        value: new_v,
                    };
                    newmap.insert(*var_index, newslot);
                }
                None => (), // this means v2 = ⊥ so v1 ∧ v2 = ⊥
            }
        }
        ArchRegsLattice { map: newmap }
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
