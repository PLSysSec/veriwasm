use crate::{analyses, checkers, ir, lattices};
use analyses::StackAnalyzer;
use analyses::{AbstractAnalyzer, AnalysisResult};
use checkers::Checker;
use ir::types::*;
use lattices::reachingdefslattice::LocIdx;
use lattices::stackgrowthlattice::StackGrowthLattice;

pub struct StackChecker<'a> {
    irmap: &'a IRMap,
    analyzer: &'a StackAnalyzer,
}

pub fn check_stack(
    result: AnalysisResult<StackGrowthLattice>,
    irmap: &IRMap,
    analyzer: &StackAnalyzer,
) -> bool {
    StackChecker {
        irmap: irmap,
        analyzer: analyzer,
    }
    .check(result)
}

impl Checker<StackGrowthLattice> for StackChecker<'_> {
    fn check(&self, result: AnalysisResult<StackGrowthLattice>) -> bool {
        self.check_state_at_statements(result)
    }

    fn irmap(&self) -> &IRMap {
        self.irmap
    }
    fn aexec(&self, state: &mut StackGrowthLattice, ir_stmt: &Stmt, loc: &LocIdx) {
        self.analyzer.aexec(state, ir_stmt, loc)
    }

    fn check_statement(
        &self,
        state: &StackGrowthLattice,
        ir_stmt: &Stmt,
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
                if dst.is_stack_access() {
                    if !self.check_stack_write(state, dst) {
                        log::debug!(
                            "check_stack_write failed: access = {:?} state = {:?}",
                            dst,
                            state
                        );
                        return false;
                    }
                }
                if dst.is_frame_access() {
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
                if src.is_stack_access() {
                    if !self.check_stack_read(state, src) {
                        log::debug!(
                            "check_stack_read failed: access = {:?} state = {:?}",
                            src,
                            state
                        );
                        return false;
                    }
                } else if src.is_frame_access() {
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

impl StackChecker<'_> {
    fn check_stack_read(&self, state: &StackGrowthLattice, src: &Value) -> bool {
        if let Value::Mem(_, memargs) = src {
            match memargs {
                MemArgs::Mem1Arg(_memarg) => {
                    return (-state.get_probestack().unwrap() <= state.get_stackgrowth().unwrap())
                        && (state.get_stackgrowth().unwrap() < 8096)
                }
                MemArgs::Mem2Args(_memarg1, memarg2) => {
                    let offset = memarg2.to_imm();
                    return (-state.get_probestack().unwrap()
                        <= state.get_stackgrowth().unwrap() + offset)
                        && (state.get_stackgrowth().unwrap() + offset < 8096);
                }
                _ => return false, //stack accesses should never have 3 args
            }
        }
        panic!("Unreachable")
    }

    fn check_bp_read(&self, state: &StackGrowthLattice, src: &Value) -> bool {
        if let Value::Mem(_, memargs) = src {
            match memargs {
                MemArgs::Mem1Arg(_memarg) => {
                    return (-state.get_probestack().unwrap() <= state.get_rbp().unwrap())
                        && (state.get_rbp().unwrap() < 8096)
                }
                MemArgs::Mem2Args(_memarg1, memarg2) => {
                    let offset = memarg2.to_imm();
                    return (-state.get_probestack().unwrap() <= state.get_rbp().unwrap() + offset)
                        && (state.get_rbp().unwrap() + offset < 8096);
                }
                _ => return false, //stack accesses should never have 3 args
            }
        }
        panic!("Unreachable")
    }

    fn check_stack_write(&self, state: &StackGrowthLattice, dst: &Value) -> bool {
        if let Value::Mem(_, memargs) = dst {
            match memargs {
                MemArgs::Mem1Arg(_memarg) => {
                    return (-state.get_probestack().unwrap() <= state.get_stackgrowth().unwrap())
                        && (state.get_stackgrowth().unwrap() < 0);
                }
                MemArgs::Mem2Args(_memarg1, memarg2) => {
                    let offset = memarg2.to_imm();
                    return (-state.get_probestack().unwrap()
                        <= state.get_stackgrowth().unwrap() + offset)
                        && (state.get_stackgrowth().unwrap() + offset < 0);
                }
                _ => return false, //stack accesses should never have 3 args
            }
        }
        panic!("Unreachable")
    }

    fn check_bp_write(&self, state: &StackGrowthLattice, dst: &Value) -> bool {
        if let Value::Mem(_, memargs) = dst {
            match memargs {
                MemArgs::Mem1Arg(_memarg) => {
                    return (-state.get_probestack().unwrap() <= state.get_rbp().unwrap())
                        && (state.get_rbp().unwrap() < 0);
                }
                MemArgs::Mem2Args(_memarg1, memarg2) => {
                    let offset = memarg2.to_imm();
                    return (-state.get_probestack().unwrap() <= state.get_rbp().unwrap() + offset)
                        && (state.get_rbp().unwrap() + offset < 0);
                }
                _ => return false, //stack accesses should never have 3 args
            }
        }
        panic!("Unreachable")
    }
}
