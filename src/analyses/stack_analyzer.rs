use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::stackgrowthlattice::StackGrowthLattice;
use crate::analyses::{AbstractAnalyzer};
use crate::lifter::{Stmt, Binopcode};
use crate::ir_utils::{is_rsp, get_imm_offset};

pub struct StackAnalyzer{}

impl AbstractAnalyzer<StackGrowthLattice> for StackAnalyzer {
    fn init_state(&self) -> StackGrowthLattice {
        StackGrowthLattice::new((0,4096))
    }

    fn aexec(&self, in_state : &mut StackGrowthLattice, ir_instr : &Stmt, _loc_idx : &LocIdx) -> () {
        // println!("Stack aexec: {:x} : {:?} rsp = {:?}", loc_idx.addr, ir_instr, in_state.v);
        match ir_instr{
            Stmt::Clear(dst, _) => if is_rsp(dst){*in_state = Default::default()},
            Stmt::Unop(_, dst, _) => if is_rsp(dst){*in_state = Default::default()},
            Stmt::Binop(Binopcode::Cmp, _, _, _) => (),
            Stmt::Binop(Binopcode::Test, _, _, _) => (),
            Stmt::Binop(opcode, dst, src1, src2) =>{  
            if is_rsp(dst) {
                if is_rsp(src1){ 
                    let offset = get_imm_offset(src2);
                    if let Some((x,probestack)) = in_state.v{
                        match opcode{
                            Binopcode::Add => {/*println!("{:?} += {:?}",x, offset);*/ *in_state = StackGrowthLattice {v : Some ((x + offset, probestack))}},
                            Binopcode::Sub => {/*println!("{:?} -= {:?}",x, offset);*/
                                if (offset - x) > probestack + 4096{
                                    panic!("Probestack violation")
                                } 
                                else if (offset - x) > probestack{
                                    //if we touch next page after the space
                                    //we've probed, it cannot skip guard page
                                    *in_state = StackGrowthLattice {v : Some ((x - offset, probestack + 4096))};
                                    return;
                                }
                                *in_state = StackGrowthLattice {v : Some ((x - offset, probestack))}
                            },
                            _ => panic!("Illegal RSP write")
                        }
                    }
                    else {*in_state = Default::default() }
                }
                else{*in_state = Default::default() }
            }
            },
            Stmt::ProbeStack(new_probestack) => 
            if let Some((x,_old_probestack)) = in_state.v{
                let probed = (((*new_probestack / 4096) + 1) * 4096) as i64; // Assumes page size of 4096
                *in_state = StackGrowthLattice {v : Some ((x - *new_probestack as i64, probed)) }
            }
            else {*in_state = Default::default() },
            _ => ()
        }
    }
}

