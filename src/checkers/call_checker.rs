use crate::analyses::call_analyzer::CallAnalyzer;
use crate::lifter::IRMap;
use crate::checkers::Checker;
use crate::analyses::{AnalysisResult};
use crate::lattices::calllattice::CallCheckLattice;

pub struct CallChecker<'a>{
    irmap : &'a  IRMap, 
    analyzer : &'a CallAnalyzer
}

pub fn check_calls(result : AnalysisResult<CallCheckLattice>,
    irmap : &IRMap, 
    analyzer : &CallAnalyzer) -> bool{
    CallChecker{irmap, analyzer}.check(result)    
}

impl Checker<CallCheckLattice> for CallChecker<'_> {
    fn check(&self, result : AnalysisResult<CallCheckLattice>) -> bool{
        self.check_state_at_blocks(result)
    }

    fn check_state(&self, state : &CallCheckLattice) -> bool {
        true
    }
}

