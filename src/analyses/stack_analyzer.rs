use crate::{analyses, ir, lattices};
use analyses::AbstractAnalyzer;
use ir::types::{Binopcode, RegT, Stmt, Unopcode, Value};
use ir::utils::get_imm_offset;
use lattices::reachingdefslattice::LocIdx;
use lattices::stackgrowthlattice::StackGrowthLattice;

pub struct StackAnalyzer {}

fn sg_lattice(stackgrowth: i64, probestack: i64, rbp: i64) -> StackGrowthLattice {
    StackGrowthLattice::new((stackgrowth, probestack, rbp))
}

impl StackAnalyzer {
    fn aeval_binop<Ar: RegT>(
        &self,
        in_state: &mut StackGrowthLattice,
        opcode: &Binopcode,
        src1: &Value<Ar>,
        src2: &Value<Ar>,
    ) -> StackGrowthLattice {
        if src1.is_rsp() {
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

impl<Ar: RegT> AbstractAnalyzer<Ar, StackGrowthLattice> for StackAnalyzer {
    fn init_state(&self) -> StackGrowthLattice {
        StackGrowthLattice::new((0, 4096, 0))
    }

    fn aexec(
        &self,
        in_state: &mut StackGrowthLattice,
        ir_instr: &Stmt<Ar>,
        loc_idx: &LocIdx,
    ) -> () {
        match ir_instr {
            Stmt::Clear(dst, _) => {
                if dst.is_rsp() {
                    *in_state = Default::default();
                    // if in_state.v.is_none(){
                    //     panic!("1. No known stack growth: 0x{:x}: {:?}", loc_idx.addr, ir_instr)
                    // }
                }
            }
            Stmt::Unop(Unopcode::Mov, dst, src) if dst.is_rsp() && src.is_rbp() => {
                if let Some((_, probestack, rbp_sg)) = in_state.v {
                    *in_state = sg_lattice(rbp_sg, probestack, rbp_sg);
                }
            }
            Stmt::Unop(Unopcode::Mov, dst, src) if dst.is_rbp() && src.is_rsp() => {
                if let Some((stackgrowth, probestack, _)) = in_state.v {
                    *in_state = sg_lattice(stackgrowth, probestack, stackgrowth);
                }
            }
            Stmt::Unop(_, dst, _) => {
                if dst.is_rsp() {
                    *in_state = Default::default();
                    // if in_state.v.is_none(){
                    //     panic!("2. No known stack growth: 0x{:x}: {:?}", loc_idx.addr, ir_instr)
                    // }
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
                if dst.is_rsp() {
                    *in_state = self.aeval_binop(in_state, opcode, src1, src2);
                    // if in_state.v.is_none(){
                    //     println!("{:?} {:?} {:?} {:?}", src1, src2, src1.is_rsp(), in_state.v);
                    //     panic!("3. No known stack growth: 0x{:x}: {:?}", loc_idx.addr, ir_instr)
                    // }
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
        if in_state.v.is_none() {
            panic!(
                "No known stack growth: 0x{:x}: {:?}",
                loc_idx.addr, ir_instr
            )
        }
    }
}
