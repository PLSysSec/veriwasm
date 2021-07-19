use crate::{analyses, ir, lattices};
use analyses::AbstractAnalyzer;
use ir::types::{Binopcode, Stmt, Unopcode, Value};
use ir::utils::{get_imm_offset, is_rbp, is_rsp};
use lattices::reachingdefslattice::LocIdx;
use lattices::stackgrowthlattice::StackGrowthLattice;

pub struct StackAnalyzer {}

fn sg_lattice(stackgrowth: i64, probestack: i64, rbp: i64) -> StackGrowthLattice{
    StackGrowthLattice::new((stackgrowth, probestack, rbp))
}

impl StackAnalyzer{
    fn aeval_binop(&self, in_state: &mut StackGrowthLattice, opcode: &Binopcode, src1: &Value, src2: &Value) -> StackGrowthLattice{
        if is_rsp(src1) {
            let offset = get_imm_offset(src2);
            if let Some((x, probestack, rbp)) = in_state.v {
                match opcode {
                    Binopcode::Add => {
                        return sg_lattice(x + offset, probestack, rbp);
                    }
                    Binopcode::Sub => {
                        if (offset - x) > probestack + 4096 {
                            panic!("Probestack violation")
                        } else if (offset - x) > probestack {
                            //if we touch next page after the space
                            //we've probed, it cannot skip guard page
                            return sg_lattice(x - offset, probestack + 4096, rbp);
                        }
                        return sg_lattice(x - offset, probestack, rbp);
                    }
                    _ => panic!("Illegal RSP write"),
                }
            } 
        } 
        Default::default()
    }
}


impl AbstractAnalyzer<StackGrowthLattice> for StackAnalyzer {
    fn init_state(&self) -> StackGrowthLattice {
        StackGrowthLattice::new((0, 4096, 0))
    }


    fn aexec(&self, in_state: &mut StackGrowthLattice, ir_instr: &Stmt, loc_idx: &LocIdx) -> () {
        match ir_instr {
            Stmt::Clear(dst, _) => {
                if is_rsp(dst) {
                    *in_state = Default::default()
                }
            }
            Stmt::Unop(Unopcode::Mov, dst, src) if is_rsp(dst) && is_rbp(src) => {
                if let Some((_, probestack, rbp_sg)) = in_state.v {
                    *in_state = sg_lattice(rbp_sg, probestack, rbp_sg);

                }
            }
            Stmt::Unop(Unopcode::Mov, dst, src) if is_rbp(dst) && is_rsp(src) => {
                if let Some((stackgrowth, probestack, _)) = in_state.v {
                    *in_state = sg_lattice(stackgrowth, probestack, stackgrowth);
                }
            }
            Stmt::Unop(_, dst, _) => {
                if is_rsp(dst) {
                    *in_state = Default::default()
                }
            }
            Stmt::Binop(Binopcode::Cmp, _, _, _) => (),
            Stmt::Binop(Binopcode::Test, _, _, _) => (),
            Stmt::Binop(opcode, dst, src1, src2) => {
                log::debug!(
                "Processing stack instruction: 0x{:x} {:?}",
                loc_idx.addr,
                ir_instr
                );
                if is_rsp(dst) {                    
                    *in_state = self.aeval_binop(in_state, opcode, src1, src2);
                }
            }
            Stmt::ProbeStack(new_probestack) => {
                if let Some((x, _old_probestack, rbp)) = in_state.v {
                    let probed = (((*new_probestack / 4096) + 1) * 4096) as i64; // Assumes page size of 4096
                    *in_state = sg_lattice(x - *new_probestack as i64, probed, rbp);
                } else {
                    *in_state = Default::default()
                }
            }
            _ => (),
        }
    }
}
