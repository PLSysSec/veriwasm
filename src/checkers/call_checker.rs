// use crate::{analyses, checkers, ir, lattices};
// use analyses::{AbstractAnalyzer, AnalysisResult, CallAnalyzer};
// use checkers::Checker;
// use ir::types::{IRMap, MemArg, MemArgs, Stmt, ValSize, Value, RegT};
// use lattices::calllattice::{CallCheckLattice, CallCheckValue};
// use lattices::davlattice::DAV;
// use lattices::reachingdefslattice::LocIdx;
// use crate::ir::types::X86Regs;

// use CallCheckValue::*;
// use ValSize::*;

// pub struct CallChecker<'a, Ar: RegT> {
//     irmap: &'a IRMap<Ar>,
//     analyzer: &'a CallAnalyzer<Ar>,
//     funcs: &'a Vec<u64>,
//     plt: &'a (u64, u64),
// }

// pub fn check_calls<Ar: RegT>(
//     result: AnalysisResult<CallCheckLattice<Ar>>,
//     irmap: &IRMap<Ar>,
//     analyzer: &CallAnalyzer<Ar>,
//     funcs: &Vec<u64>,
//     plt: &(u64, u64),
// ) -> bool {
//     CallChecker {
//         irmap,
//         analyzer,
//         funcs,
//         plt,
//     }
//     .check(result)
// }

// impl<Ar: RegT> Checker<Ar, CallCheckLattice<Ar>> for CallChecker<'_, Ar> {
//     fn check(&self, result: AnalysisResult<CallCheckLattice<Ar>>) -> bool {
//         self.check_state_at_statements(result)
//     }

//     fn irmap(&self) -> &IRMap<Ar> {
//         self.irmap
//     }
//     fn aexec(&self, state: &mut CallCheckLattice<Ar>, ir_stmt: &Stmt<Ar>, loc: &LocIdx) {
//         self.analyzer.aexec(state, ir_stmt, loc)
//     }

//     fn check_statement(&self, state: &CallCheckLattice<Ar>, ir_stmt: &Stmt<Ar>, loc_idx: &LocIdx) -> bool {
//         //1. Check that all indirect calls use resolved function pointer
//         if let Stmt::Call(v) = ir_stmt {
//             if !self.check_indirect_call(state, v, loc_idx) {
//                 println!("0x{:x} Failure Case: Indirect Call {:?}", loc_idx.addr, v);
//                 return false;
//             }
//         }

//         // 2. Check that lookup is using resolved DAV
//         if let Stmt::Unop(_, _, Value::Mem(_, memargs)) = ir_stmt {
//             if !self.check_calltable_lookup(state, memargs) {
//                 println!(
//                     "0x{:x} Failure Case: Lookup Call: {:?}",
//                     loc_idx.addr, memargs
//                 );
//                 print_mem_access(state, memargs);
//                 return false;
//             }
//         }
//         true
//     }
// }

// impl<Ar: RegT> CallChecker<'_, Ar> {
//     fn check_indirect_call(
//         &self,
//         state: &CallCheckLattice<Ar>,
//         target: &Value<Ar>,
//         loc_idx: &LocIdx,
//     ) -> bool {
//         match target {
//             Value::Reg(regnum, size) => {
//                 if let Some(FnPtr(c)) = state.regs.get_reg(*regnum, *size).v {
//                     return true;
//                 } else {
//                     log::debug!("{:?}", state.regs.get_reg(*regnum, *size).v)
//                 }
//             }
//             Value::Mem(_, _) => return false,
//             Value::Imm(_, _, imm) => {
//                 let target = (*imm + (loc_idx.addr as i64) + 5) as u64;
//                 let (plt_start, plt_end) = self.plt;
//                 return self.funcs.contains(&target)
//                     || ((target >= *plt_start) && (target < *plt_end));
//             }
//             Value::RIPConst => {
//                 return true;
//             }
//         }
//         false
//     }

//     fn check_calltable_lookup(&self, state: &CallCheckLattice<Ar>, memargs: &MemArgs<Ar>) -> bool {
//         log::debug!("Call Table Lookup: {:?}", memargs);
//         match memargs {
//             MemArgs::Mem3Args(
//                 MemArg::Reg(regnum1, Size64),
//                 MemArg::Reg(regnum2, Size64),
//                 MemArg::Imm(_, _, 8),
//             ) => match (
//                 state.regs.get_reg(*regnum1, Size64).v,
//                 state.regs.get_reg(*regnum2, Size64).v,
//             ) {
//                 (Some(GuestTableBase), Some(PtrOffset(DAV::Checked))) => return true,
//                 (Some(PtrOffset(DAV::Checked)), Some(GuestTableBase)) => return true,
//                 (Some(TypedPtrOffset(_)), Some(GuestTableBase)) => return true,
//                 (Some(GuestTableBase), Some(TypedPtrOffset(_))) => return true,
//                 (_x, Some(GuestTableBase)) | (Some(GuestTableBase), _x) => return false,
//                 (_x, _y) => return true, // not a calltable lookup
//             },
//             _ => return true, //not a calltable lookup?
//         }
//     }
// }

// pub fn memarg_repr<Ar: RegT>(state: &CallCheckLattice<Ar>, memarg: &MemArg<Ar>) -> String {
//     match memarg {
//         MemArg::Reg(regnum, size) => {
//             format!("r{:?}: {:?}", regnum, state.regs.get_reg(*regnum, *size).v)
//         }
//         MemArg::Imm(_, _, x) => format!("{:?}", x),
//     }
// }

// pub fn print_mem_access<Ar: RegT>(state: &CallCheckLattice<Ar>, memargs: &MemArgs<Ar>) {
//     match memargs {
//         MemArgs::Mem1Arg(x) => log::debug!("mem[{:?}]", memarg_repr(state, x)),
//         MemArgs::Mem2Args(x, y) => log::debug!(
//             "mem[{:?} + {:?}]",
//             memarg_repr(state, x),
//             memarg_repr(state, y)
//         ),
//         MemArgs::Mem3Args(x, y, z) => log::debug!(
//             "mem[{:?} + {:?} + {:?}]",
//             memarg_repr(state, x),
//             memarg_repr(state, y),
//             memarg_repr(state, z)
//         ),
//         MemArgs::MemScale(x, y, z) => log::debug!(
//             "mem[{:?} + {:?} * {:?}]",
//             memarg_repr(state, x),
//             memarg_repr(state, y),
//             memarg_repr(state, z)
//         ),
//     }
// }
