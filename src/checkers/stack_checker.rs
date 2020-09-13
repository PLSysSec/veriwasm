use crate::lifter::IRMap;
use crate::analyses::stack_analyzer::StackAnalyzer;
use crate::checkers::Checker;
use crate::analyses::{AnalysisResult};
use crate::lattices::stackgrowthlattice::StackGrowthLattice;

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


