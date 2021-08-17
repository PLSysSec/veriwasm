use crate::ir::types::RegT;
use crate::{analyses, ir, lattices};
use analyses::AnalysisResult;
use ir::types::{IRMap, Stmt};
use lattices::reachingdefslattice::LocIdx;
use lattices::Lattice;
use itertools::Itertools;

mod call_checker;
mod heap_checker;
mod jump_resolver;
pub mod locals_checker;
mod stack_checker;
mod wasmtime_checker;

/*      Public API for checker submodule      */
// pub use self::call_checker::check_calls;
pub use self::heap_checker::check_heap;
// pub use self::jump_resolver::resolve_jumps;
pub use self::stack_checker::check_stack;
pub use self::wasmtime_checker::check_wasmtime;

pub trait Checker<Ar: RegT, State: Lattice + Clone> {
    fn check(&self, result: AnalysisResult<State>) -> bool;
    fn irmap(&self) -> &IRMap<Ar>;
    fn aexec(&self, state: &mut State, ir_stmt: &Stmt<Ar>, loc: &LocIdx);

    fn check_state_at_statements(&self, result: AnalysisResult<State>) -> bool {
        // TODO: only do in sorted order when debug is enabled
        for block_addr in result.keys().sorted() {
            let mut state = result[block_addr].clone();
            log::debug!(
                "Checking block 0x{:x} with start state {:?}",
                block_addr,
                state
            );
            let block = self.irmap().get(&block_addr).unwrap();
            for (addr, ir_stmts) in block {
                for (idx, ir_stmt) in ir_stmts.iter().enumerate() {
                    log::debug!(
                        "Checking stmt at 0x{:x}: {:?} with start state {:?}",
                        addr,
                        ir_stmt,
                        state
                    );
                    let loc_idx = LocIdx {
                        addr: *addr,
                        idx: idx as u32,
                    };
                    if !self.check_statement(&state, ir_stmt, &loc_idx) {
                        return false;
                    }
                    self.aexec(&mut state, ir_stmt, &loc_idx);
                }
            }
        }
        true
    }
    fn check_statement(&self, state: &State, ir_stmt: &Stmt<Ar>, loc_idx: &LocIdx) -> bool;
}
