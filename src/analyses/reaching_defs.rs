use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::reachingdefslattice::{ReachLattice, singleton, LocIdx};
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Stmt};
use crate::utils::{LucetMetadata};
use std::default::Default;

//Top level function
pub fn analyze_reaching_defs(cfg : &ControlFlowGraph<u64>, irmap : &IRMap, _metadata : LucetMetadata){
    run_worklist(cfg, irmap, ReachingDefnAnalyzer{});    
}

pub struct ReachingDefnAnalyzer{
}

impl AbstractAnalyzer<ReachLattice> for ReachingDefnAnalyzer {
    fn init_state(&self) -> ReachLattice {
        Default::default()
    }

    fn aexec(&self, in_state : &mut ReachLattice, ir_instr : &Stmt, loc_idx : &LocIdx) -> () {
        match ir_instr{
            Stmt::Clear(dst) => in_state.set(dst, singleton(loc_idx.clone())),
            Stmt::Unop(_, dst, _) =>  in_state.set(dst, singleton(loc_idx.clone())),
            Stmt::Binop(_, dst, src1, src2) =>  {
                in_state.adjust_stack_offset(dst, src1, src2);  
                in_state.set(dst, singleton(loc_idx.clone()))
            },
            Stmt::Call(_) => in_state.regs.clear_regs(),
            _ => ()
        }
    }
}
