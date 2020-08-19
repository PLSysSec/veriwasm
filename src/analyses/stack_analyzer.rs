use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::stackgrowthlattice::StackGrowthLattice;
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Stmt, Value};

pub fn analyze_stack(cfg : &ControlFlowGraph<u64>, irmap : IRMap){
    run_worklist(cfg, irmap, StackAnalyzer{});    
}

pub struct StackAnalyzer{
}

impl AbstractAnalyzer<StackGrowthLattice> for StackAnalyzer {
    fn init_state(&self) -> StackGrowthLattice {
        StackGrowthLattice {v : Some(0)}
    }

    //TODO: how to get size of pop / push
    //TODO: binop => allow for PLUS and MINUS stack adjustments
    fn aexec(&self, in_state : &mut StackGrowthLattice, ir_instr : &Stmt) -> () {
        match ir_instr{
            Stmt::Clear(_, dst) => 
            if let Value::Reg(regnum,_) = dst {
                if *regnum == 4 {
                    *in_state = StackGrowthLattice {v : None};
                }     
            },
            Stmt::Unop(_, dst, _) => 
            if let Value::Reg(regnum,_) = dst {
                if *regnum == 4 {
                    *in_state = StackGrowthLattice {v : None};
                }     
            },
            Stmt::Binop(_, dst, _, _) =>  
            if let Value::Reg(regnum,_) = dst {
                if *regnum == 4 {
                    *in_state = StackGrowthLattice {v : None};
                }     
            },
            Stmt::Call(_) => (),
            Stmt::Pop(_) => *in_state = StackGrowthLattice {v : in_state.v.map(|x| x + 8)}, 
            Stmt::Push(_) => *in_state = StackGrowthLattice {v : in_state.v.map(|x| x - 8)},
            _ => ()
        }
    }

    fn process_branch(&self, in_state : StackGrowthLattice) -> Vec<StackGrowthLattice>{
        // let output : Vec<StackGrowthLattice> = {in_state.clone(), in_state.clone()}
        let mut output = Vec::new();
        output.push(in_state.clone());
        output.push(in_state.clone());
        output
    }
}

