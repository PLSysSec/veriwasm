pub mod stack_analyzer;
pub mod heap_analyzer;
pub mod call_analyzer;
pub mod jump_analyzer;
pub mod reaching_defs;
use yaxpeax_core::analyses::control_flow::VW_CFG;
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::{Lattice, VarState};
use std::collections::{HashMap, VecDeque};
use crate::lifter::{IRMap, IRBlock, Stmt, Binopcode,Value};


pub type AnalysisResult<T>  = HashMap<u64, T>;

pub trait AbstractAnalyzer<State:Lattice + VarState + Clone> {
    fn init_state(&self) -> State{Default::default()}
    fn process_branch(&self, _irmap : &IRMap, in_state : &State, succ_addrs : &Vec<u64>, _addr : &u64) -> Vec<(u64,State)>{
        succ_addrs.into_iter().map(|addr| (addr.clone(),in_state.clone()) ).collect()
    }
    fn aexec_unop(&self, in_state : &mut State, dst : &Value, _src : &Value, _loc_idx : &LocIdx) -> (){
        in_state.set_to_bot(dst)
    }
    fn aexec_binop(&self, in_state : &mut State, opcode : &Binopcode, dst: &Value, _src1 : &Value, _src2: &Value, _loc_idx : &LocIdx) -> (){
        match opcode{
            Binopcode::Cmp => (),
            Binopcode::Test => (),
            _ => in_state.set_to_bot(dst)
        }
    }

    fn aexec(&self, in_state : &mut State, ir_instr : &Stmt, loc_idx : &LocIdx) -> (){
        match ir_instr{
            Stmt::Clear(dst, _srcs) => in_state.set_to_bot(dst),
            Stmt::Unop(_, dst, src) => self.aexec_unop(in_state, &dst, &src, loc_idx),
            Stmt::Binop(opcode, dst, src1, src2) =>  {self.aexec_binop(in_state, opcode, dst, src1, src2, loc_idx); in_state.adjust_stack_offset(opcode,dst,src1,src2)},
            Stmt::Call(_) => in_state.on_call(),
            _ => ()
        }
    }
}

fn analyze_block<T:AbstractAnalyzer<State>, State:VarState + Lattice + Clone> (analyzer : &T, state : &State, irblock : &IRBlock) -> State {
    let mut new_state = state.clone();
    for (addr,instruction) in irblock.iter(){
        for (idx,ir_insn) in instruction.iter().enumerate(){
            // if *addr== 0x111a || *addr == 0x01167{
            //     println!(">>> {:x} {:?}", addr, state);
            // }
            analyzer.aexec(&mut new_state, ir_insn, &LocIdx {addr : *addr, idx : idx as u32});
        }
    }
    new_state
}

fn align_succ_addrs(addr : u64, succ_addrs : Vec<u64>) -> Vec<u64>{
    if succ_addrs.len() != 2{
        return succ_addrs;
    }
    let a1 = succ_addrs[0];
    let a2 = succ_addrs[1];
    if a1 < addr {return vec![a2,a1];}
    if a2 < addr {return vec![a1,a2];}
    if a1 < a2 {return vec![a1, a2];}
    if a1 >= a2 {return vec![a2, a1];}
    panic!("Unreachable");

}

pub fn run_worklist<T:AbstractAnalyzer<State>, State:VarState + Lattice + Clone> (cfg : &VW_CFG, irmap : &IRMap, analyzer : &T) -> AnalysisResult<State>{
    let mut statemap : HashMap<u64, State> = HashMap::new();
    let mut worklist: VecDeque<u64> = VecDeque::new();
    worklist.push_back(cfg.entrypoint);
    statemap.insert(cfg.entrypoint, analyzer.init_state());
    
    while !worklist.is_empty(){
        let addr = worklist.pop_front().unwrap();
        let irblock = irmap.get(&addr).unwrap();
        let state = statemap.get(&addr).unwrap(); 
        let new_state = analyze_block(analyzer, state, irblock);
        let succ_addrs_unaligned : Vec<u64> = cfg.graph.neighbors(addr).collect();
        let succ_addrs : Vec<u64> = align_succ_addrs(addr, succ_addrs_unaligned);
        //println!("Processing Block: 0x{:x} -> {:?}", addr, succ_addrs);
        for (succ_addr,branch_state) in analyzer.process_branch(irmap, &new_state, &succ_addrs, &addr){
            let mut has_change = false;
            if statemap.contains_key(&succ_addr){
                let old_state = statemap.get(&succ_addr).unwrap();
                let merged_state = old_state.meet(&branch_state, &LocIdx {addr : addr, idx : 0});   
                // if succ_addr == 0x001055bb{
                //     println!(">>>Meet: 0x{:x} => 0x{:x} merging {:?} into {:?}", addr, succ_addr, branch_state, old_state);
                //     println!("merged_state = {:?}", merged_state);
                // }
                if merged_state > *old_state {
                    println!("{:?} {:?}", merged_state, old_state);
                    panic!("Meet monoticity error");
                }
                //has_change = *old_state != branch_state;
                has_change = *old_state != merged_state;
                statemap.insert(succ_addr, merged_state);
            }
            else{
                //TODO: pretty sure this is an unnecessary state clone
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
