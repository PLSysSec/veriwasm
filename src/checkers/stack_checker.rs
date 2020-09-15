use crate::ir_utils::is_stack_access;
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lifter::{Stmt,Value};
use crate::lifter::IRMap;
use crate::analyses::stack_analyzer::StackAnalyzer;
use crate::checkers::Checker;
use crate::analyses::{AnalysisResult};
use crate::lattices::stackgrowthlattice::StackGrowthLattice;
use crate::analyses::AbstractAnalyzer;

//TODO: how to iterate over statements as opposed to blocks?

pub struct StackChecker<'a>{
    irmap : &'a  IRMap, 
    analyzer : &'a StackAnalyzer
}

pub fn check_stack(result : AnalysisResult<StackGrowthLattice>, 
    irmap : &IRMap, 
    analyzer : &StackAnalyzer) -> bool{
    StackChecker{irmap : irmap, analyzer : analyzer}.check(result)    
}

impl Checker<StackGrowthLattice> for StackChecker<'_> {
    fn check(&self, result : AnalysisResult<StackGrowthLattice>) -> bool{
        self.check_state_at_blocks(result)
    }

    fn check_state(&self, state : &StackGrowthLattice) -> bool {
        match state.v {
            None => return false,
            Some(stackgrowth) => if stackgrowth >= 0 {
                return false
            } 
        }
        true
    }
}



impl StackChecker<'_> {
    fn check_state_at_statements(&self, result : AnalysisResult<StackGrowthLattice>) -> bool{
        for (block_addr,mut state) in result {
            for (addr,ir_stmts) in self.irmap.get(&block_addr).unwrap(){
                for (idx,ir_stmt) in ir_stmts.iter().enumerate(){
                    self.analyzer.aexec(&mut state, ir_stmt, &LocIdx {addr : *addr, idx : idx as u32});
                    if !self.check_statement(&state, ir_stmt){
                        return false
                    }
                }
            }
        }
        true
    }

    // TODO Complete
    fn check_statement(&self, state : &StackGrowthLattice, ir_stmt : &Stmt) -> bool {
        //1, stackgrowth is never Bottom or >= 0
        match state.v {
            None => return false,
            Some(stackgrowth) => if stackgrowth >= 0 {
                return false
            } 
        }

        //2. Reads and writes are in bounds
        match ir_stmt {
            //encapsulates both load and store
            Stmt::Unop(_,dst,src) => 
            // stack write: probestack <= stackgrowth + c < 0
            if is_stack_access(dst){
                ()
            }
            //stack read: probestack <= stackgrowth + c < 8K
            else if is_stack_access(src) {
                ()
            },
            _ => (),
        }
        
        // 3. For all rets stackgrowth = 0
        if let Stmt::Ret = ir_stmt{
            if let Some(stackgrowth) = state.v {
                if stackgrowth != 0 {
                    return false
                }
            }
        }
        
        true
    }
}


