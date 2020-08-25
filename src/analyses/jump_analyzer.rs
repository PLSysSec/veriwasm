use crate::analyses::AnalysisResult;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::switchlattice::{SwitchLattice};
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lattices::reachingdefslattice::{ReachLattice, LocIdx};
use crate::lifter::{IRMap, Stmt};
use crate::utils::{LucetMetadata};
use std::default::Default;

//Top level function
pub fn analyze_jumps(cfg : &ControlFlowGraph<u64>, irmap : IRMap, metadata : LucetMetadata, reaching_defs : AnalysisResult<ReachLattice>){
    run_worklist(cfg, irmap, SwitchAnalyzer{metadata : metadata, reaching_defs : reaching_defs});    
}

pub struct SwitchAnalyzer{
    metadata: LucetMetadata,
    reaching_defs : AnalysisResult<ReachLattice>
}

impl AbstractAnalyzer<SwitchLattice> for SwitchAnalyzer {
    fn init_state(&self) -> SwitchLattice {
        Default::default()
    }

    // TODO: complete this aexec function
    fn aexec(&self, in_state : &mut SwitchLattice, ir_instr : &Stmt, loc_idx : &LocIdx) -> () {
        match ir_instr{
            Stmt::Clear(dst) => in_state.set_to_bot(dst),
            Stmt::Unop(_, dst, src) => in_state.set_to_bot(dst),
            Stmt::Binop(_, dst, src1, src2) =>  in_state.default_exec_binop(dst,src1,src2),
            Stmt::Call(_) => in_state.regs.clear_regs(),
            _ => ()
        }
    }
}
