use crate::lifter::{Stmt, Value};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::analyses::call_analyzer::CallAnalyzer;
use crate::lifter::IRMap;
use crate::checkers::Checker;
use crate::analyses::{AnalysisResult};
use crate::lattices::calllattice::CallCheckLattice;
use crate::analyses::AbstractAnalyzer;
use crate::lattices;

pub struct CallChecker<'a>{
    irmap : &'a  IRMap, 
    analyzer : &'a CallAnalyzer
}

// TODO
// if not has indirect_calls:
//      return true     
pub fn check_calls(result : AnalysisResult<CallCheckLattice>,
    irmap : &IRMap, 
    analyzer : &CallAnalyzer) -> bool{
    CallChecker{irmap, analyzer}.check(result)    
}

impl Checker<CallCheckLattice> for CallChecker<'_> {
    fn check(&self, result : AnalysisResult<CallCheckLattice>) -> bool{
        self.check_state_at_statements(result)
    }

    fn check_state(&self, state : &CallCheckLattice) -> bool {
        true
    }
}

impl CallChecker<'_> {
    fn check_state_at_statements(&self, result : AnalysisResult<CallCheckLattice>) -> bool{
        for (block_addr,mut state) in result {
            for (addr,ir_stmts) in self.irmap.get(&block_addr).unwrap(){
                for (idx,ir_stmt) in ir_stmts.iter().enumerate(){
                    self.analyzer.aexec(&mut state, ir_stmt, &LocIdx {addr : *addr, idx : idx as u32});
                    if !self.check_state(&state){
                        return false
                    }
                }
            }
        }
        true
    }
    // TODO Complete
    fn check_statement(&self, state : &CallCheckLattice, ir_stmt : &Stmt) -> bool {
        if let Stmt::Call(value) = ir_stmt{
            match value {
                Value::Reg(regnum, size) => return false,
                _ => ()//return true
            }
        }
        
        true
    }
}


