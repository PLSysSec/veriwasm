use crate::{analyses, ir, lattices};
use analyses::AnalysisResult;
use ir::types::{IRMap, Stmt};
use lattices::reachingdefslattice::LocIdx;
use lattices::Lattice;

mod call_checker;
mod heap_checker;
mod jump_resolver;
pub mod locals_checker;
mod stack_checker;

/*      Public API for checker submodule      */
pub use self::call_checker::check_calls;
pub use self::heap_checker::check_heap;
pub use self::jump_resolver::resolve_jumps;
pub use self::stack_checker::check_stack;

pub trait Checker<State: Lattice + Clone> {
    fn check(&self, result: AnalysisResult<State>) -> bool;
    fn irmap(&self) -> &IRMap;
    fn aexec(&self, state: &mut State, ir_stmt: &Stmt, loc: &LocIdx);

    fn check_state_at_statements(&self, result: AnalysisResult<State>) -> bool {
        for (block_addr, mut state) in result {
            log::debug!(
                "Checking block 0x{:x} with start state {:?}",
                block_addr,
                state
            );
            for (addr, ir_stmts) in self.irmap().get(&block_addr).unwrap() {
                for (idx, ir_stmt) in ir_stmts.iter().enumerate() {
                    log::debug!(
                        "Checking stmt at 0x{:x}: {:?} with start state {:?}",
                        addr,
                        ir_stmt,
                        state
                    );
                    if !self.check_statement(
                        &state,
                        ir_stmt,
                        &LocIdx {
                            addr: *addr,
                            idx: idx as u32,
                        },
                    ) {
                        return false;
                    }
                    self.aexec(
                        &mut state,
                        ir_stmt,
                        &LocIdx {
                            addr: *addr,
                            idx: idx as u32,
                        },
                    );
                }
            }
        }
        true
    }
    fn check_statement(&self, state: &State, ir_stmt: &Stmt, loc_idx: &LocIdx) -> bool;
}
