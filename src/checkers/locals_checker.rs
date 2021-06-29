use std::convert::TryFrom;
use std::collections::HashSet;

use crate::analyses::locals_analyzer::LocalsAnalyzer;
use crate::analyses::{AbstractAnalyzer, AnalysisResult};
use crate::checkers::Checker;
use crate::ir::types::{IRMap, MemArgs, Stmt, Value};
use crate::lattices::X86Regs;
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::localslattice::{LocalsLattice, SlotVal};

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
    LocalsChecker {
        irmap,
        analyzer,
    }
    .check(result)
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

    fn check_statement(
        &self,
        state: &LocalsLattice,
        stmt: &Stmt,
        loc_idx: &LocIdx,
    ) -> bool {
        let debug_addrs : HashSet<u64> = vec![0x000122b7].into_iter().collect();
        if debug_addrs.contains(&loc_idx.addr) {
            println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
            println!("check_statement debug 0x{:x?}: {:?}\n{:?}", loc_idx.addr, stmt, state);
            let mut cloned = state.clone();
            self.aexec(&mut cloned, stmt, loc_idx);
            println!("{:?}", cloned);
            println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        }
        match stmt {
            Stmt::Clear(dst, srcs) => {
                if self.analyzer.aeval_vals(state, srcs) == Uninit {
                    match dst {
                        Value::Mem(memsize, memargs) => {},
                        Value::Reg(reg_num, _) => {
                            if *reg_num != u8::from(Rsp) && *reg_num != u8::from(Zf) {
                                println!("----------------------------------------");
                                println!("{:?}", state);
                                println!("Darn: 0x{:x?}: {:?}", loc_idx.addr, stmt);
                                println!("reg: {:?}", X86Regs::try_from(*reg_num));
                                println!("type: {:?}", self.analyzer.fun_type);
                                println!("----------------------------------------")
                                // return false;
                            }
                        },
                        Value::Imm(_, _, _) => {},
                        Value::RIPConst => todo!(),
                    }
                }
            },
            _ => {}
        }
        // println!("{:?}", state);
        // println!("0x{:x?}: {:?}", loc_idx.addr, stmt);

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

