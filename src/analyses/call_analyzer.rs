use crate::lattices::calllattice::CallCheckLattice;
use crate::lifter::{MemArgs, MemArg, ValSize, Binopcode, IRMap, Stmt, Value};
use crate::utils::get_rsp_offset;
use crate::analyses::AnalysisResult;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lattices::reachingdefslattice::{ReachLattice, LocIdx};
use crate::utils::{LucetMetadata};
use std::default::Default;

//Top level function
pub fn analyze_calls(cfg : &ControlFlowGraph<u64>, irmap : &IRMap, metadata : LucetMetadata, reaching_defs : AnalysisResult<ReachLattice>){
    run_worklist(cfg, irmap, CallAnalyzer{metadata : metadata, reaching_defs : reaching_defs});    
}

pub struct CallAnalyzer{
    metadata: LucetMetadata,
    reaching_defs : AnalysisResult<ReachLattice>
}

impl AbstractAnalyzer<CallCheckLattice> for CallAnalyzer {
    fn init_state(&self) -> CallCheckLattice {
        Default::default()
    }

    //TODO: complete this aexec function
    fn aexec(&self, in_state : &mut CallCheckLattice, ir_instr : &Stmt, loc_idx : &LocIdx) -> () {
        match ir_instr{
            Stmt::Clear(dst) => in_state.set_to_bot(dst),
            Stmt::Unop(_, dst, src) => in_state.set_to_bot(dst),//in_state.set(dst, self.aeval_unop(in_state, src)),
            Stmt::Binop(opcode, dst, src1, src2) =>  in_state.default_exec_binop(dst,src1,src2),
            Stmt::Call(_) => in_state.regs.clear_regs(),
            _ => ()
        }
    }



}

