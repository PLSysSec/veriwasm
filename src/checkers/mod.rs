use crate::lattices::reachingdefslattice::LocIdx;
use crate::lifter::Stmt;
use crate::analyses::AbstractAnalyzer;
use crate::lifter::IRMap;
use crate::analyses::AnalysisResult;
use crate::lattices::Lattice;

pub mod stack_checker;
pub mod heap_checker;
pub mod call_checker;


pub trait Checker<State:Lattice + Clone> {
    fn check(&self, result : AnalysisResult<State>) -> bool;
    fn irmap(&self) -> &IRMap;
    fn aexec(&self, state: &mut State, ir_stmt: &Stmt, loc: &LocIdx);

    fn check_state_at_statements(&self, result : AnalysisResult<State>) -> bool{
        for (block_addr,mut state) in result {
            for (addr,ir_stmts) in self.irmap().get(&block_addr).unwrap(){
                for (idx,ir_stmt) in ir_stmts.iter().enumerate(){
                    println!("------------\n{:x} {:?}", addr, ir_stmt);
                    if !self.check_statement(&state, ir_stmt){
                        return false
                    }
                    self.aexec(&mut state, ir_stmt, &LocIdx {addr : *addr, idx : idx as u32});
                }
            }
        }
        true
    }
    fn check_statement(&self, state : &State, ir_stmt : &Stmt) -> bool;
}