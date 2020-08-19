// pub mod lifter;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::stackgrowthlattice::StackGrowthLattice;
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Stmt};

pub fn analyze_stack(cfg : &ControlFlowGraph<u64>, irmap : IRMap){
    println!("Nice");
    run_worklist(cfg, irmap, StackAnalyzer{});
    
}



pub struct StackAnalyzer{
}

impl AbstractAnalyzer<StackGrowthLattice> for StackAnalyzer {
    // type State = StackGrowthLattice;
    fn init_state(&self) -> StackGrowthLattice {
        StackGrowthLattice {v : Some(0)}
    }

    fn aexec(&self, in_state : StackGrowthLattice, ir_instr : Stmt) -> StackGrowthLattice {
        // match ir_instr{
        //     Clear(yaxpeax_x86::long_mode::Opcode, Value),
        //     Unop(yaxpeax_x86::long_mode::Opcode, Value, Value),
        //     Binop(yaxpeax_x86::long_mode::Opcode, Value, Value, Value),
        //     Call(Value),
        //     Pop(Value),
        //     Push(Value),
        //     _ => ()
        // }
        unimplemented!()
    }

    fn process_branch(&self, in_state : StackGrowthLattice) -> (Vec<StackGrowthLattice>){
        // let output : Vec<StackGrowthLattice> = {in_state.clone(), in_state.clone()}
        let mut output = Vec::new();
        output.push(in_state.clone());
        output.push(in_state.clone());
        output
    }
}

