use crate::lifter::IRMap;
use crate::analyses::AnalysisResult;
use crate::lattices::Lattice;

pub mod stack_checker;
pub mod heap_checker;
pub mod call_checker;


pub trait Checker<State:Lattice + Clone> {
    fn check(&self, result : AnalysisResult<State>) -> bool;
    fn check_state_at_blocks(&self, result : AnalysisResult<State>) -> bool{
        for (block_addr,state) in result {
            return self.check_state(&state)
            }
        //empty irmap is safe I guess
        true
    }
    // fn check_state_at_statements(&self, result : AnalysisResult<State>, irmap : &IRMap, analyzer : T) -> bool{
    //     for (block_addr,state) in result {
    //         return self.check_state(&state)
    //         }
    //     //empty irmap is safe I guess
    //     true
    // }
    fn check_state(&self, state : &State) -> bool;
}