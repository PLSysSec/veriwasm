pub mod stack_analyzer;
pub mod heap_analyzer;
pub mod call_analyzer;
pub mod jump_analyzer;
pub mod reaching_defs;
use crate::lifter::Binopcode;
use crate::lifter::Value;
use crate::lattices::VarState;
use crate::lattices::reachingdefslattice::ReachLattice;
use crate::utils::LucetMetadata;
use crate::lattices::VariableState;
use crate::lattices::reachingdefslattice::LocIdx;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::{Lattice};
use std::collections::VecDeque;
use std::collections::HashMap;
use crate::lifter::{IRMap, IRBlock, Stmt};


type AnalysisResult<T>  = HashMap<u64, T>;

pub trait AbstractAnalyzer<State:Lattice + VarState + Clone> {
    fn init_state(&self) -> State{Default::default()}
    fn process_branch(&self, in_state : &State, succ_addrs : &Vec<u64>) -> Vec<(u64,State)>{
        succ_addrs.into_iter().map(|addr| (addr.clone(),in_state.clone()) ).collect()
    }
    fn aexec_unop(&self, in_state : &mut State, dst : &Value, src : &Value) -> (){
        unimplemented!()
    }
    fn aexec_binop(&self, in_state : &mut State, opcode : &Binopcode, dst: &Value, src1 : &Value, src2: &Value) -> (){
        in_state.set_to_bot(dst)
    }

    fn aexec(&self, in_state : &mut State, ir_instr : &Stmt, loc_idx : &LocIdx) -> (){
        match ir_instr{
            Stmt::Clear(dst) => in_state.set_to_bot(dst),
            Stmt::Unop(_, dst, src) => self.aexec_unop(in_state, &dst, &src),//in_state.set(dst, self.aeval_unop(in_state, src)),
            Stmt::Binop(opcode, dst, src1, src2) =>  {self.aexec_binop(in_state, opcode, dst, src1, src2); in_state.adjust_stack_offset(dst,src1,src2)},
            Stmt::Call(_) => in_state.on_call(),
            _ => ()
        }
    }
}

fn analyze_block<T:AbstractAnalyzer<State>, State:VarState + Lattice + Clone> (analyzer : &T, state : &State, irblock : &IRBlock) -> State {
    let mut new_state = state.clone();
    for (addr,instruction) in irblock.iter(){
        for (idx,ir_insn) in instruction.iter().enumerate(){
            analyzer.aexec(&mut new_state, ir_insn, &LocIdx {addr : *addr, idx : idx as u32});
        }
    }
    new_state
}

pub fn run_worklist<T:AbstractAnalyzer<State>, State:VarState + Lattice + Clone> (cfg : &ControlFlowGraph<u64>, irmap : &IRMap, analyzer : T) -> HashMap<u64, State>{
    let mut statemap : HashMap<u64, State> = HashMap::new();
    let mut worklist: VecDeque<u64> = VecDeque::new();
    worklist.push_back(cfg.entrypoint);
    statemap.insert(cfg.entrypoint, analyzer.init_state());
    while !worklist.is_empty(){
        let addr = worklist.pop_front().unwrap();
        let irblock = irmap.get(&addr).unwrap();
        let state = statemap.get(&addr).unwrap(); 
        let new_state = analyze_block(&analyzer, state, irblock);
        let succ_addrs : Vec<u64> = cfg.graph.neighbors(addr).collect();

        for (succ_addr,branch_state) in analyzer.process_branch(&new_state, &succ_addrs){
            let mut has_change = false;
            if statemap.contains_key(&succ_addr){
                let old_state = statemap.get(&succ_addr).unwrap();
                let merged_state = old_state.meet(&branch_state);   
                if merged_state > *old_state {
                    panic!("Meet monoticity error");
                }
                has_change = *old_state != branch_state;
                statemap.insert(succ_addr, merged_state);
            }
            else{
                statemap.insert(succ_addr, branch_state.clone());
                has_change = true;
            }

            if has_change && !worklist.contains(&succ_addr){
                worklist.push_back(succ_addr);
            }
        } 
    }
    statemap
}
