pub mod stack_analyzer;
pub mod heap_analyzer;
pub mod call_analyzer;
pub mod jump_analyzer;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::{Lattice};


//abstract state is just a lattice

//<A: Arch>
//&A::Instruction

//TODO: finish analyzer
pub trait AbstractAnalyzer {
    fn init_state<T:Lattice + Clone>(&self) -> T; 
    fn aeval<T:Lattice + Clone>(&self) -> ();
    fn process_branch<T:Lattice + Clone>(&self, instate : T) -> (Vec<T>); // default implementation is one_result_for_all_successors
}

pub fn run_worklist<T:AbstractAnalyzer> (cfg : ControlFlowGraph<u64>, analyzer : T){
    unimplemented!();
}

// pub fn one_result_for_all_successors(){
//     unimplemented!();
// }
