use crate::lifter::{Stmt, Value};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::analyses::heap_analyzer::HeapAnalyzer;
use crate::lifter::IRMap;
use crate::checkers::Checker;
use crate::analyses::{AnalysisResult};
use crate::lattices::heaplattice::{HeapLattice, HeapValue};
use crate::analyses::AbstractAnalyzer;

pub struct HeapChecker<'a>{
    irmap : &'a  IRMap, 
    analyzer : &'a HeapAnalyzer
}

pub fn check_heap(result : AnalysisResult<HeapLattice>, 
    irmap : &IRMap, 
    analyzer : &HeapAnalyzer) -> bool{
    HeapChecker{irmap : irmap, analyzer : analyzer}.check(result)    
}

impl Checker<HeapLattice> for HeapChecker<'_> {
    fn check(&self, result : AnalysisResult<HeapLattice>) -> bool{
        self.check_state_at_statements(result)
    }

    //TODO: finish check_state
    fn check_state(&self, state : &HeapLattice) -> bool {
        true
    }
}

//TODO: need to update state after each statement
// impl HeapChecker<'_> {
//     fn check_state_at_statements(&self, result : AnalysisResult<HeapLattice>) -> bool{
//         for (block_addr,state) in result {
//             for (addr,ir_stmts) in self.irmap.get(&block_addr).unwrap(){
//                 for ir_stmt in ir_stmts{
//                     if !self.check_state(&state){
//                         return false
//                     }
//                 }
//             }
//         }
//         true
//     }
// }


impl HeapChecker<'_> {
    fn check_state_at_statements(&self, result : AnalysisResult<HeapLattice>) -> bool{
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
    fn check_statement(&self, state : &HeapLattice, ir_stmt : &Stmt) -> bool {
        if let Stmt::Ret = ir_stmt{
            match state.regs.rdi.v{
                Some(HeapValue::HeapBase) => return true,
                _ => ()
            }
        }
        
        true
    }
}




/*
// Check that at each call, rdi = heapbase
        if let Stmt::Ret = ir_stmt{
            match state.regs.rdi.v{
                Some(HeapValue::HeapBase) => return true,
                _ => return false
            }
        }
*/