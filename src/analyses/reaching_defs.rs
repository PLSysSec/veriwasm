use crate::analyses::{run_worklist, AbstractAnalyzer, AnalysisResult};
use crate::lattices::reachingdefslattice::{loc, singleton, LocIdx, ReachLattice};
use crate::lattices::VarState;
use crate::utils::lifter::{Binopcode, IRMap, Stmt, Unopcode};
use crate::utils::utils::LucetMetadata;
use yaxpeax_core::analyses::control_flow::VW_CFG;

//Top level function
pub fn analyze_reaching_defs(
    cfg: &VW_CFG,
    irmap: &IRMap,
    _metadata: LucetMetadata,
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

        s.regs.rax = loc(0xdeadbeef, 0);
        s.regs.rcx = loc(0xdeadbeef, 1);
        s.regs.rdx = loc(0xdeadbeef, 2);
        s.regs.rbx = loc(0xdeadbeef, 3);
        s.regs.rbp = loc(0xdeadbeef, 4);
        s.regs.rsi = loc(0xdeadbeef, 5);
        s.regs.rdi = loc(0xdeadbeef, 6);
        s.regs.r8 = loc(0xdeadbeef, 7);
        s.regs.r9 = loc(0xdeadbeef, 8);
        s.regs.r10 = loc(0xdeadbeef, 9);
        s.regs.r11 = loc(0xdeadbeef, 10);
        s.regs.r12 = loc(0xdeadbeef, 11);
        s.regs.r13 = loc(0xdeadbeef, 12);
        s.regs.r14 = loc(0xdeadbeef, 13);
        s.regs.r15 = loc(0xdeadbeef, 14);

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
            Stmt::Unop(Unopcode::Mov, dst, src) => {
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
                in_state.regs.rax = loc(loc_idx.addr, 0);
                in_state.regs.rcx = loc(loc_idx.addr, 1);
                in_state.regs.rdx = loc(loc_idx.addr, 2);
                in_state.regs.rbx = loc(loc_idx.addr, 3);
                in_state.regs.rbp = loc(loc_idx.addr, 4);
                in_state.regs.rsi = loc(loc_idx.addr, 5);
                in_state.regs.rdi = loc(loc_idx.addr, 6);
                in_state.regs.r8 = loc(loc_idx.addr, 7);
                in_state.regs.r9 = loc(loc_idx.addr, 8);
                in_state.regs.r10 = loc(loc_idx.addr, 9);
                in_state.regs.r11 = loc(loc_idx.addr, 10);
                in_state.regs.r12 = loc(loc_idx.addr, 11);
                in_state.regs.r13 = loc(loc_idx.addr, 12);
                in_state.regs.r14 = loc(loc_idx.addr, 13);
                in_state.regs.r15 = loc(loc_idx.addr, 14);
            }
            _ => (),
        }
    }
}
