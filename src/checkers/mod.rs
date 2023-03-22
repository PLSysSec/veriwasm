use crate::{analyses, ir, lattices};
use analyses::AnalysisResult;
use ir::types::*;
use itertools::Itertools;
use lattices::Lattice;

mod heap_checker;
mod jump_resolver;
mod stack_checker;

/*      Public API for checker submodule      */
pub use self::heap_checker::check_heap;
pub use self::jump_resolver::resolve_jumps;
pub use self::stack_checker::check_stack;

pub trait Checker<State: Lattice + Clone> {
    fn check(&self, result: AnalysisResult<State>) -> bool;
    fn irmap(&self) -> &IRMap;
    fn aexec(&self, state: &mut State, ir_stmt: &Stmt, loc: &LocIdx);

    fn check_state_at_statements(&self, result: AnalysisResult<State>) -> bool {
        // for (block_addr, mut state) in result {
        //     log::debug!(
        //         "Checking block 0x{:x} with start state {:?}",
        //         block_addr,
        //         state
        //     );
        for block_addr in result.keys().sorted() {
            let mut state = result[block_addr].clone();
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
    fn check_statement(&self, state: &State, ir_stmt: &Stmt, loc_idx: &LocIdx) -> bool;
}
