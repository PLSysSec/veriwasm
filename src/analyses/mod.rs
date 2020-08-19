pub mod stack_analyzer;
pub mod heap_analyzer;
pub mod call_analyzer;
pub mod jump_analyzer;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::{Lattice};
use petgraph::graphmap::GraphMap;
use std::collections::VecDeque;
use std::collections::HashMap;
use crate::lifter::{IRMap, IRBlock, Stmt};



//abstract state is just a lattice

//<A: Arch>
//&A::Instruction

//&yaxpeax_core::analyses::control_flow::ControlFlowGraph<u64>


//TODO: finish analyzer
pub trait AbstractAnalyzer<State:Lattice + Clone> {
    fn init_state(&self) -> State; 
    fn aexec(&self, in_state : State, instr : Stmt) -> State;
    fn process_branch(&self, in_state : State) -> Vec<State>;
}

// def process_irb_for_heapcheck_lattice(ircfg, irb, init: HeapCheckAbstrState):
//     cur = deepcopy(init)
//     for ab in irb.assignblks:
//         cur = process_assignblk_for_heapcheck_lattice(ab, cur)
//     # copies the current abstract state into all successors
//     return one_result_for_all_successors(ircfg.successors(irb.loc_key), cur)

//TODO: implement analyze_block
fn analyze_block<T:AbstractAnalyzer<State>, State:Lattice + Clone> (analyzer : &T, state : &State, irblock : &IRBlock) -> State {
    // let mut iter = program.instructions_spanning(<AMD64 as Arch>::Decoder::default(), block.start, block.end);
    // while let Some((address, instr)) = iter.next() {
    //     lift(instr);
    //     println!("{:?}\n", instr);
    // }
    // state
    unimplemented!();

}

//TODO: split between branches, and jumps, and many target jumps
pub fn run_worklist<T:AbstractAnalyzer<State>, State:Lattice + Clone> (cfg : &ControlFlowGraph<u64>, irmap : IRMap, analyzer : T) -> GraphMap<State, (), petgraph::Directed>{
    let mut statemap : HashMap<u64, State> = HashMap::new();
    let mut worklist: VecDeque<u64> = VecDeque::new();
    worklist.push_back(cfg.entrypoint);
    statemap.insert(cfg.entrypoint, analyzer.init_state());
    while !worklist.is_empty(){
        let addr = worklist.pop_front().unwrap();
        let irblock = irmap.get(&addr).unwrap();
        let state = statemap.get(&addr).unwrap(); 
        let new_state = analyze_block(&analyzer, state, irblock);
        for succ_addr in cfg.graph.neighbors(addr){
            let mut has_change = false;

            if statemap.contains_key(&succ_addr){
                let old_state = statemap.get(&succ_addr).unwrap();
                let merged_state = old_state.meet(&new_state);   
                if merged_state > *old_state {
                    panic!("Meet monoticity error");
                }
                has_change = *old_state != new_state;
                statemap.insert(succ_addr, merged_state);
            }
            else{
                statemap.insert(succ_addr, new_state.clone());
                has_change = true;
            }

            if has_change && !worklist.contains(&succ_addr){
                worklist.push_back(succ_addr);
            }
        } 
    }
    unimplemented!();
}

// pub fn run_worklist(){
//     unimplemented!();
// }
