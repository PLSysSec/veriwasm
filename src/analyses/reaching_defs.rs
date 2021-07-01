use crate::analyses::{run_worklist, AbstractAnalyzer, AnalysisResult};
use crate::ir::types::ValSize;
use crate::ir::types::{Binopcode, IRMap, Stmt, Unopcode, X86Regs};
use crate::lattices::reachingdefslattice::{loc, singleton, LocIdx, ReachLattice};
use crate::lattices::VarState;
use crate::loaders::utils::VW_Metadata;
use yaxpeax_core::analyses::control_flow::VW_CFG;

use X86Regs::*;

//Top level function
pub fn analyze_reaching_defs(
    cfg: &VW_CFG,
    irmap: &IRMap,
    _metadata: VW_Metadata,
) -> AnalysisResult<ReachLattice> {
    run_worklist(
        cfg,
        irmap,
        &ReachingDefnAnalyzer {
            cfg: cfg.clone(),
            irmap: irmap.clone(),
        },
    )
}

pub struct ReachingDefnAnalyzer {
    pub cfg: VW_CFG,
    pub irmap: IRMap,
}

impl ReachingDefnAnalyzer {
    //1. get enclosing block addr
    //2. get result for that block start
    //3. run reaching def up to that point
    pub fn fetch_def(
        &self,
        result: &AnalysisResult<ReachLattice>,
        loc_idx: &LocIdx,
    ) -> ReachLattice {
        if self.cfg.blocks.contains_key(&loc_idx.addr) {
            return result.get(&loc_idx.addr).unwrap().clone();
        }
        let block_addr = self.cfg.prev_block(loc_idx.addr).unwrap().start;
        let irblock = self.irmap.get(&block_addr).unwrap();
        let mut def_state = result.get(&block_addr).unwrap().clone();
        for (addr, instruction) in irblock.iter() {
            for (idx, ir_insn) in instruction.iter().enumerate() {
                if &loc_idx.addr == addr && (loc_idx.idx as usize) == idx {
                    return def_state;
                }
                self.aexec(
                    &mut def_state,
                    ir_insn,
                    &LocIdx {
                        addr: *addr,
                        idx: idx as u32,
                    },
                );
            }
        }
        unimplemented!()
    }
}

impl AbstractAnalyzer<ReachLattice> for ReachingDefnAnalyzer {
    fn init_state(&self) -> ReachLattice {
        let mut s: ReachLattice = Default::default();

        s.regs.set_reg(Rax, ValSize::Size64, loc(0xdeadbeef, 0));
        s.regs.set_reg(Rcx, ValSize::Size64, loc(0xdeadbeef, 1));
        s.regs.set_reg(Rdx, ValSize::Size64, loc(0xdeadbeef, 2));
        s.regs.set_reg(Rbx, ValSize::Size64, loc(0xdeadbeef, 3));
        s.regs.set_reg(Rbp, ValSize::Size64, loc(0xdeadbeef, 4));
        s.regs.set_reg(Rsi, ValSize::Size64, loc(0xdeadbeef, 5));
        s.regs.set_reg(Rdi, ValSize::Size64, loc(0xdeadbeef, 6));
        s.regs.set_reg(R8, ValSize::Size64, loc(0xdeadbeef, 7));
        s.regs.set_reg(R9, ValSize::Size64, loc(0xdeadbeef, 8));
        s.regs.set_reg(R10, ValSize::Size64, loc(0xdeadbeef, 9));
        s.regs.set_reg(R11, ValSize::Size64, loc(0xdeadbeef, 10));
        s.regs.set_reg(R12, ValSize::Size64, loc(0xdeadbeef, 11));
        s.regs.set_reg(R13, ValSize::Size64, loc(0xdeadbeef, 12));
        s.regs.set_reg(R14, ValSize::Size64, loc(0xdeadbeef, 13));
        s.regs.set_reg(R15, ValSize::Size64, loc(0xdeadbeef, 14));

        s.stack.update(0x8, loc(0xdeadbeef, 15), 4);
        s.stack.update(0x10, loc(0xdeadbeef, 16), 4);
        s.stack.update(0x18, loc(0xdeadbeef, 17), 4);
        s.stack.update(0x20, loc(0xdeadbeef, 18), 4);
        s.stack.update(0x28, loc(0xdeadbeef, 18), 4);

        s
    }

    fn aexec(&self, in_state: &mut ReachLattice, ir_instr: &Stmt, loc_idx: &LocIdx) -> () {
        match ir_instr {
            Stmt::Clear(dst, _) => in_state.set(dst, singleton(loc_idx.clone())),
            Stmt::Unop(Unopcode::Mov, dst, src) | Stmt::Unop(Unopcode::Movsx, dst, src) => {
                if let Some(v) = in_state.get(src) {
                    if v.defs.is_empty() {
                        in_state.set(dst, singleton(loc_idx.clone()));
                    } else {
                        in_state.set(dst, v);
                    }
                } else {
                    in_state.set(dst, singleton(loc_idx.clone()));
                }
                //in_state.set(dst, singleton(loc_idx.clone()))
            }
            Stmt::Binop(Binopcode::Cmp, _, _, _) => {
                //Ignore compare
            }
            Stmt::Binop(Binopcode::Test, _, _, _) => {
                //Ignore test
            }
            Stmt::Binop(opcode, dst, src1, src2) => {
                in_state.adjust_stack_offset(opcode, dst, src1, src2);
                in_state.set(dst, singleton(loc_idx.clone()))
            }
            Stmt::Call(_) => {
                in_state
                    .regs
                    .set_reg(Rax, ValSize::Size64, loc(loc_idx.addr, 0));
                in_state
                    .regs
                    .set_reg(Rcx, ValSize::Size64, loc(loc_idx.addr, 1));
                in_state
                    .regs
                    .set_reg(Rdx, ValSize::Size64, loc(loc_idx.addr, 2));
                in_state
                    .regs
                    .set_reg(Rbx, ValSize::Size64, loc(loc_idx.addr, 3));
                in_state
                    .regs
                    .set_reg(Rbp, ValSize::Size64, loc(loc_idx.addr, 4));
                in_state
                    .regs
                    .set_reg(Rsi, ValSize::Size64, loc(loc_idx.addr, 5));
                in_state
                    .regs
                    .set_reg(Rdi, ValSize::Size64, loc(loc_idx.addr, 6));
                in_state
                    .regs
                    .set_reg(R8, ValSize::Size64, loc(loc_idx.addr, 7));
                in_state
                    .regs
                    .set_reg(R9, ValSize::Size64, loc(loc_idx.addr, 8));
                in_state
                    .regs
                    .set_reg(R10, ValSize::Size64, loc(loc_idx.addr, 9));
                in_state
                    .regs
                    .set_reg(R11, ValSize::Size64, loc(loc_idx.addr, 10));
                in_state
                    .regs
                    .set_reg(R12, ValSize::Size64, loc(loc_idx.addr, 11));
                in_state
                    .regs
                    .set_reg(R13, ValSize::Size64, loc(loc_idx.addr, 12));
                in_state
                    .regs
                    .set_reg(R14, ValSize::Size64, loc(loc_idx.addr, 13));
                in_state
                    .regs
                    .set_reg(R15, ValSize::Size64, loc(loc_idx.addr, 14));
            }
            _ => (),
        }
    }
}
