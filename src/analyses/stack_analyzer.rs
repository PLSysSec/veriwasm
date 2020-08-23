use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::stackgrowthlattice::StackGrowthLattice;
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Stmt, Value};

pub fn analyze_stack(cfg : &ControlFlowGraph<u64>, irmap : IRMap){
    run_worklist(cfg, irmap, StackAnalyzer{});    
}

pub struct StackAnalyzer{}

impl AbstractAnalyzer<StackGrowthLattice> for StackAnalyzer {
    fn init_state(&self) -> StackGrowthLattice {
        StackGrowthLattice {v : Some(0)}
    }

    fn aexec(&self, in_state : &mut StackGrowthLattice, ir_instr : &Stmt) -> () {
        match ir_instr{
            Stmt::Clear(dst) => 
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
            Stmt::Binop(_, dst, src1, src2) =>  
            if let Value::Reg(regnum,size) = dst {
                if *regnum == 4 {
                    assert_eq!(size.to_u32(), 64);
                    match(src1, src2){
                        (Value::Reg(regnum2,size2),Value::Imm(_,_,v)) =>{ 
                            if *regnum2 == 4{
                                assert_eq!(size2.to_u32(), 64);
                                *in_state = StackGrowthLattice {v : in_state.v.map(|x| x + v)}
                            }
                        }
                        _ => panic!("Illegal RSP write")
                    }
                }     
            },
            Stmt::Call(_) => (),
            _ => ()
        }
    }
}

