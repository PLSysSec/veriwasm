use crate::analyses::heap_analyzer::HeapAnalyzer;
use crate::lifter::IRMap;
use crate::checkers::Checker;
use crate::analyses::{AnalysisResult};
use crate::lattices::heaplattice::HeapLattice;

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
        self.check_state_at_blocks(result)
    }

    fn check_state(&self, state : &HeapLattice) -> bool {
        true
    }
}