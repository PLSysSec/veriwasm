use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::reachingdefslattice::{ReachLattice, singleton};
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Stmt};
use crate::utils::{LucetMetadata, get_rsp_offset};
use std::default::Default;

//Top level function
pub fn analyze_reaching_defs(cfg : &ControlFlowGraph<u64>, irmap : IRMap, metadata : LucetMetadata){
    run_worklist(cfg, irmap, ReachingDefnAnalyzer{});    
}

pub struct ReachingDefnAnalyzer{
}

impl AbstractAnalyzer<ReachLattice> for ReachingDefnAnalyzer {
    fn init_state(&self) -> ReachLattice {
        Default::default()
    }

    // TODO - handle stack offset tracking
    fn aexec(&self, in_state : &mut ReachLattice, ir_instr : &Stmt, addr : &u64) -> () {
        match ir_instr{
            Stmt::Clear(dst) => in_state.set(dst, singleton(addr.clone())),
            Stmt::Unop(_, dst, _) =>  in_state.set(dst, singleton(addr.clone())),
            Stmt::Binop(_, dst, _, _) =>  in_state.set(dst, singleton(addr.clone())),
            Stmt::Call(_) => in_state.regs.clear_regs(),
            _ => ()
        }
    }
}
