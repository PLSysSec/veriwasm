use crate::{analyses, ir, lattices};
use analyses::AbstractAnalyzer;
use ir::types::*;
use lattices::reachingdefslattice::LocIdx;
use lattices::stackgrowthlattice::StackGrowthLattice;

pub struct StackAnalyzer {}

impl AbstractAnalyzer<StackGrowthLattice> for StackAnalyzer {
    fn init_state(&self) -> StackGrowthLattice {
        StackGrowthLattice::new((0, 4096, 0))
    }

    fn aexec(&self, in_state: &mut StackGrowthLattice, ir_instr: &Stmt, loc_idx: &LocIdx) -> () {
        match ir_instr {
            Stmt::Clear(dst, _) => {
                if dst.is_rsp() {
                    *in_state = Default::default()
                }
            }
            Stmt::Unop(Unopcode::Mov, dst, src) if dst.is_rsp() && src.is_rbp() => {
                if let Some((_, probestack, rbp_stackgrowth)) = in_state.v {
                    *in_state = StackGrowthLattice {
                        v: Some((rbp_stackgrowth, probestack, rbp_stackgrowth)),
                    };
                }
            }
            Stmt::Unop(Unopcode::Mov, dst, src) if dst.is_rbp() && src.is_rsp() => {
                if let Some((stackgrowth, probestack, _)) = in_state.v {
                    *in_state = StackGrowthLattice {
                        v: Some((stackgrowth, probestack, stackgrowth)),
                    };
                }
            }
            Stmt::Unop(_, dst, _) => {
                if dst.is_rsp() {
                    *in_state = Default::default()
                }
            }
            Stmt::Binop(Binopcode::Cmp, _, _, _) => (),
            Stmt::Binop(Binopcode::Test, _, _, _) => (),
            Stmt::Binop(opcode, dst, src1, src2) => {
                if dst.is_rsp() {
                    if src1.is_rsp() {
                        log::debug!(
                            "Processing stack instruction: 0x{:x} {:?}",
                            loc_idx.addr,
                            ir_instr
                        );
                        let offset = src2.as_imm_val();
                        if let Some((x, probestack, rbp)) = in_state.v {
                            match opcode {
                                Binopcode::Add => {
                                    *in_state = StackGrowthLattice {
                                        v: Some((x + offset, probestack, rbp)),
                                    }
                                }
                                Binopcode::Sub => {
                                    if (offset - x) > probestack + 4096 {
                                        panic!("Probestack violation")
                                    } else if (offset - x) > probestack {
                                        //if we touch next page after the space
                                        //we've probed, it cannot skip guard page
                                        *in_state = StackGrowthLattice {
                                            v: Some((x - offset, probestack + 4096, rbp)),
                                        };
                                        return;
                                    }
                                    *in_state = StackGrowthLattice {
                                        v: Some((x - offset, probestack, rbp)),
                                    }
                                }
                                _ => panic!("Illegal RSP write"),
                            }
                        } else {
                            *in_state = Default::default()
                        }
                    } else {
                        *in_state = Default::default()
                    }
                }
            }
            Stmt::ProbeStack(new_probestack) => {
                if let Some((x, _old_probestack, rbp)) = in_state.v {
                    let probed = (((*new_probestack / 4096) + 1) * 4096) as i64; // Assumes page size of 4096
                    *in_state = StackGrowthLattice {
                        v: Some((x - *new_probestack as i64, probed, rbp)),
                    }
                } else {
                    *in_state = Default::default()
                }
            }
            _ => (),
        }
    }
}
