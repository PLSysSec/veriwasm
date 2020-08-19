// pub mod lifter;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::stackgrowthlattice::StackGrowthLattice;
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Stmt};

pub fn analyze_stack(cfg : &ControlFlowGraph<u64>, irmap : IRMap){
    println!("Nice");
    run_worklist(cfg, irmap, StackAnalyzer{});
    
}

// fn update_stackgrowth(state : &mut StackGrowthLattice, adjustment : i64){
//     state.v.map(|x| x + adjustment)
// }


pub struct StackAnalyzer{
}

impl AbstractAnalyzer<StackGrowthLattice> for StackAnalyzer {
    // type State = StackGrowthLattice;
    fn init_state(&self) -> StackGrowthLattice {
        StackGrowthLattice {v : Some(0)}
    }

    //TODO: how to get size of pop / push
    fn aexec(&self, in_state : &mut StackGrowthLattice, ir_instr : &Stmt) -> () {
        // match ir_instr{
        //     Clear(opcode, dst) => (),
        //     Unop(opcode, dst, arg) => (),
        //     Binop(opcode, dst, arg1, arg2) => (),
        //     Call(target) => (),
        //     Pop(dst) => StackGrowthLattice {v : in_state.v.map(|x| x + 8)}, 
        //     Push(dst) => StackGrowthLattice {v : in_state.v.map(|x| x - 8)},
        // }
        unimplemented!()
    }

    fn process_branch(&self, in_state : StackGrowthLattice) -> Vec<StackGrowthLattice>{
        // let output : Vec<StackGrowthLattice> = {in_state.clone(), in_state.clone()}
        let mut output = Vec::new();
        output.push(in_state.clone());
        output.push(in_state.clone());
        output
    }
}

