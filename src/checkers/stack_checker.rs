use crate::{analyses, checkers, ir, lattices};
use analyses::StackAnalyzer;
use analyses::{AbstractAnalyzer, AnalysisResult};
use checkers::Checker;
use ir::types::{IRMap, MemArgs, Stmt, Value};
use ir::utils::{get_imm_mem_offset, is_bp_access, is_stack_access};
use lattices::reachingdefslattice::LocIdx;
use lattices::stackgrowthlattice::StackGrowthLattice;
use crate::ir::types::X86Regs;
use crate::ir::types::RegT;

pub struct StackChecker<'a, Ar> {
    irmap: &'a IRMap<Ar>,
    analyzer: &'a StackAnalyzer,
}

pub fn check_stack<Ar: RegT>(
    result: AnalysisResult<StackGrowthLattice>,
    irmap: &IRMap<Ar>,
    analyzer: &StackAnalyzer,
) -> bool {
    StackChecker {
        irmap: irmap,
        analyzer: analyzer,
    }
    .check(result)
}

impl<Ar: RegT> Checker<Ar, StackGrowthLattice> for StackChecker<'_, Ar> {
    fn check(&self, result: AnalysisResult<StackGrowthLattice>) -> bool {
        self.check_state_at_statements(result)
    }

    fn irmap(&self) -> &IRMap<Ar> {
        self.irmap
    }
    fn aexec(&self, state: &mut StackGrowthLattice, ir_stmt: &Stmt<Ar>, loc: &LocIdx) {
        self.analyzer.aexec(state, ir_stmt, loc)
    }

    fn check_statement(
        &self,
        state: &StackGrowthLattice,
        ir_stmt: &Stmt<Ar>,
        _loc_idx: &LocIdx,
    ) -> bool {
        //1, stackgrowth is never Bottom or >= 0
        match state.v {
            None => {
                println!("Failure Case at {:?}: Stackgrowth = None", ir_stmt);
                return false;
            }
            Some((stackgrowth, _, _)) => {
                if stackgrowth > 0 {
                    return false;
                }
            }
        }

        // 2. Reads and writes are in bounds
        match ir_stmt {
            //encapsulates both load and store
            Stmt::Unop(_, dst, src) =>
            // stack write: probestack <= stackgrowth + c < 0
            {
                if is_stack_access(dst) {
                    if !self.check_stack_write(state, dst) {
                        log::debug!(
                            "check_stack_write failed: access = {:?} state = {:?}",
                            dst,
                            state
                        );
                        return false;
                    }
                }
                if is_bp_access(dst) {
                    if !self.check_bp_write(state, dst) {
                        log::debug!(
                            "check_bp_write failed: access = {:?} state = {:?}",
                            dst,
                            state
                        );
                        return false;
                    }
                }
                //stack read: probestack <= stackgrowth + c < 8K
                if is_stack_access(src) {
                    if !self.check_stack_read(state, src) {
                        log::debug!(
                            "check_stack_read failed: access = {:?} state = {:?}",
                            src,
                            state
                        );
                        return false;
                    }
                } else if is_bp_access(src) {
                    if !self.check_bp_read(state, src) {
                        log::debug!(
                            "check_bp_read failed: access = {:?} state = {:?}",
                            src,
                            state
                        );
                        return false;
                    }
                }
            }
            _ => (),
        }

        // 3. For all rets stackgrowth = 0
        if let Stmt::Ret = ir_stmt {
            if let Some((stackgrowth, _, _)) = state.v {
                if stackgrowth != 0 {
                    log::debug!("stackgrowth != 0 at ret: stackgrowth = {:?}", stackgrowth);
                    return false;
                }
            }
        }

        true
    }
}

impl<Ar: RegT> StackChecker<'_, Ar> {
    fn check_stack_read(&self, state: &StackGrowthLattice, src: &Value<Ar>) -> bool {
        if let Value::Mem(_, memargs) = src {
            match memargs {
                MemArgs::Mem1Arg(_memarg) => {
                    return (-state.get_probestack().unwrap() <= state.get_stackgrowth().unwrap())
                        && (state.get_stackgrowth().unwrap() < 8096)
                }
                MemArgs::Mem2Args(_memarg1, memarg2) => {
                    let offset = get_imm_mem_offset(memarg2);
                    return (-state.get_probestack().unwrap()
                        <= state.get_stackgrowth().unwrap() + offset)
                        && (state.get_stackgrowth().unwrap() + offset < 8096);
                }
                _ => return false, //stack accesses should never have 3 args
            }
        }
        panic!("Unreachable")
    }

    fn check_bp_read(&self, state: &StackGrowthLattice, src: &Value<Ar>) -> bool {
        if let Value::Mem(_, memargs) = src {
            match memargs {
                MemArgs::Mem1Arg(_memarg) => {
                    return (-state.get_probestack().unwrap() <= state.get_rbp().unwrap())
                        && (state.get_rbp().unwrap() < 8096)
                }
                MemArgs::Mem2Args(_memarg1, memarg2) => {
                    let offset = get_imm_mem_offset(memarg2);
                    return (-state.get_probestack().unwrap() <= state.get_rbp().unwrap() + offset)
                        && (state.get_rbp().unwrap() + offset < 8096);
                }
                _ => return false, //stack accesses should never have 3 args
            }
        }
        panic!("Unreachable")
    }

    fn check_stack_write(&self, state: &StackGrowthLattice, dst: &Value<Ar>) -> bool {
        if let Value::Mem(_, memargs) = dst {
            match memargs {
                MemArgs::Mem1Arg(_memarg) => {
                    return (-state.get_probestack().unwrap() <= state.get_stackgrowth().unwrap())
                        && (state.get_stackgrowth().unwrap() < 0);
                }
                MemArgs::Mem2Args(_memarg1, memarg2) => {
                    let offset = get_imm_mem_offset(memarg2);
                    return (-state.get_probestack().unwrap()
                        <= state.get_stackgrowth().unwrap() + offset)
                        && (state.get_stackgrowth().unwrap() + offset < 0);
                }
                _ => return false, //stack accesses should never have 3 args
            }
        }
        panic!("Unreachable")
    }

    fn check_bp_write(&self, state: &StackGrowthLattice, dst: &Value<Ar>) -> bool {
        if let Value::Mem(_, memargs) = dst {
            match memargs {
                MemArgs::Mem1Arg(_memarg) => {
                    return (-state.get_probestack().unwrap() <= state.get_rbp().unwrap())
                        && (state.get_rbp().unwrap() < 0);
                }
                MemArgs::Mem2Args(_memarg1, memarg2) => {
                    let offset = get_imm_mem_offset(memarg2);
                    return (-state.get_probestack().unwrap() <= state.get_rbp().unwrap() + offset)
                        && (state.get_rbp().unwrap() + offset < 0);
                }
                _ => return false, //stack accesses should never have 3 args
            }
        }
        panic!("Unreachable")
    }
}
