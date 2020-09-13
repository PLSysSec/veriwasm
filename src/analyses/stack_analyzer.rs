use crate::lattices::reachingdefslattice::LocIdx;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::stackgrowthlattice::StackGrowthLattice;
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Stmt};
use crate::ir_utils::{is_rsp, get_imm_offset};
use crate::analyses::AnalysisResult;

// pub fn analyze_stack(cfg : &ControlFlowGraph<u64>, irmap : &IRMap) -> AnalysisResult<StackGrowthLattice>{
//     run_worklist(cfg, &irmap, StackAnalyzer{})    
// }

pub struct StackAnalyzer{}

impl AbstractAnalyzer<StackGrowthLattice> for StackAnalyzer {
    fn init_state(&self) -> StackGrowthLattice {
        StackGrowthLattice::new(0)
    }

    fn aexec(&self, in_state : &mut StackGrowthLattice, ir_instr : &Stmt, _loc_idx : &LocIdx) -> () {
        println!("Stack aexec: {:?} @ {:?}", ir_instr, _loc_idx);
        match ir_instr{
            Stmt::Clear(dst) => if is_rsp(dst){*in_state = Default::default()},
            Stmt::Unop(_, dst, _) => if is_rsp(dst){*in_state = Default::default()},
            Stmt::Binop(_, dst, src1, src2) =>  
            if is_rsp(dst) {
                if is_rsp(src1){ 
                    let offset = get_imm_offset(src2);
                    *in_state = StackGrowthLattice {v : in_state.v.map(|x| x + offset)} 
                }
                else{ panic!("Illegal RSP write") }
            },
            _ => ()
        }
    }
}

