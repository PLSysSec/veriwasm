use std::collections::HashSet;
use std::convert::TryFrom;

use crate::analyses::locals_analyzer::LocalsAnalyzer;
use crate::analyses::{AbstractAnalyzer, AnalysisResult};
use crate::checkers::Checker;
use crate::ir::types::{IRMap, MemArgs, Stmt, Value, X86Regs};
use crate::lattices::localslattice::{LocalsLattice, SlotVal};
use crate::lattices::reachingdefslattice::LocIdx;

use SlotVal::*;
use X86Regs::*;

pub struct LocalsChecker<'a> {
    irmap: &'a IRMap,
    analyzer: &'a LocalsAnalyzer<'a>,
}

pub fn check_locals(
    result: AnalysisResult<LocalsLattice>,
    irmap: &IRMap,
    analyzer: &LocalsAnalyzer,
) -> bool {
    LocalsChecker { irmap, analyzer }.check(result)
}

impl Checker<LocalsLattice> for LocalsChecker<'_> {
    fn check(&self, result: AnalysisResult<LocalsLattice>) -> bool {
        self.check_state_at_statements(result)
    }

    fn irmap(&self) -> &IRMap {
        self.irmap
    }

    fn aexec(&self, state: &mut LocalsLattice, ir_stmt: &Stmt, loc: &LocIdx) {
        self.analyzer.aexec(state, ir_stmt, loc)
    }

    fn check_statement(&self, state: &LocalsLattice, stmt: &Stmt, loc_idx: &LocIdx) -> bool {
        let debug_addrs: HashSet<u64> = vec![].into_iter().collect();
        if debug_addrs.contains(&loc_idx.addr) {
            println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
            println!("{:?}", state);
            println!("check_statement debug 0x{:x?}: {:?}", loc_idx.addr, stmt);
            let mut cloned = state.clone();
            self.aexec(&mut cloned, stmt, loc_idx);
            println!("{:?}", cloned);
            println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        }
        let error = match stmt {
            Stmt::Clear(dst, srcs) => {
                if self.analyzer.aeval_vals(state, srcs, loc_idx) == Uninit {
                    match dst {
                        Value::Mem(memsize, memargs) => true,
                        Value::Reg(reg_num, _) => {
                            *reg_num != Rsp && *reg_num != Zf && *reg_num != Cf && *reg_num != Rbp
                        }
                        Value::Imm(_, _, _) => false,
                        Value::RIPConst => false,
                    }
                } else {
                    false
                }
            }
            Stmt::Unop(_, dst, src) => {
                if self.analyzer.aeval_val(state, src, loc_idx) == Uninit {
                    match dst {
                        Value::Mem(memsize, memargs) => true,
                        Value::Reg(reg_num, _) => {
                            *reg_num != Rsp && *reg_num != Zf && *reg_num != Cf && *reg_num != Rbp
                        }
                        Value::Imm(_, _, _) => false,
                        Value::RIPConst => false,
                    }
                } else {
                    false
                }
            }
            Stmt::Binop(opcode, dst, src1, src2) => {
                if self
                    .analyzer
                    .aeval_vals(state, &vec![src1.clone(), src2.clone()], loc_idx)
                    == Uninit
                {
                    match dst {
                        Value::Mem(memsize, memargs) => true,
                        Value::Reg(reg_num, _) => {
                            *reg_num != Rsp && *reg_num != Zf && *reg_num != Cf && *reg_num != Rbp
                        }
                        Value::Imm(_, _, _) => false,
                        Value::RIPConst => false,
                    }
                } else {
                    false
                }
            }
            Stmt::Branch(br_type, val) => self.analyzer.aeval_val(state, val, loc_idx) == Uninit,
            _ => false,
        };
        if error {
            println!("----------------------------------------");
            println!("{:?}", state);
            println!("Darn: 0x{:x?}: {:?}", loc_idx.addr, stmt);
            println!("----------------------------------------")
        }

        // //1, stackgrowth is never Bottom or >= 0
        // match state.v {
        //     None => {
        //         println!("Failure Case at {:?}: Stackgrowth = None", ir_stmt);
        //         return false;
        //     }
        //     Some((stackgrowth, _, _)) => {
        //         if stackgrowth > 0 {
        //             return false;
        //         }
        //     }
        // }

        // // 2. Reads and writes are in bounds
        // match ir_stmt {
        //     //encapsulates both load and store
        //     Stmt::Unop(_, dst, src) =>
        //     // stack write: probestack <= stackgrowth + c < 0
        //     {
        //         if is_stack_access(dst) {
        //             if !self.check_stack_write(state, dst) {
        //                 log::debug!(
        //                     "check_stack_write failed: access = {:?} state = {:?}",
        //                     dst,
        //                     state
        //                 );
        //                 return false;
        //             }
        //         }
        //         if is_bp_access(dst) {
        //             if !self.check_bp_write(state, dst) {
        //                 log::debug!(
        //                     "check_bp_write failed: access = {:?} state = {:?}",
        //                     dst,
        //                     state
        //                 );
        //                 return false;
        //             }
        //         }
        //         //stack read: probestack <= stackgrowth + c < 8K
        //         if is_stack_access(src) {
        //             if !self.check_stack_read(state, src) {
        //                 log::debug!(
        //                     "check_stack_read failed: access = {:?} state = {:?}",
        //                     src,
        //                     state
        //                 );
        //                 return false;
        //             }
        //         } else if is_bp_access(src) {
        //             if !self.check_bp_read(state, src) {
        //                 log::debug!(
        //                     "check_bp_read failed: access = {:?} state = {:?}",
        //                     src,
        //                     state
        //                 );
        //                 return false;
        //             }
        //         }
        //     }
        //     _ => (),
        // }

        // // 3. For all rets stackgrowth = 0
        // if let Stmt::Ret = ir_stmt {
        //     if let Some((stackgrowth, _, _)) = state.v {
        //         if stackgrowth != 0 {
        //             log::debug!("stackgrowth != 0 at ret: stackgrowth = {:?}", stackgrowth);
        //             return false;
        //         }
        //     }
        // }

        true
    }
}
