use crate::analyses::reaching_defs::ReachingDefnAnalyzer;
use crate::analyses::{run_worklist, AbstractAnalyzer, AnalysisResult};
use crate::ir::types::{Binopcode, IRMap, MemArg, MemArgs, Unopcode, ValSize, Value, X86Regs};
use crate::ir::utils::get_rsp_offset;
use crate::lattices::reachingdefslattice::{LocIdx, ReachLattice};
use crate::lattices::VarSlot;
use crate::lattices::switchlattice::{SwitchLattice, SwitchValue, SwitchValueLattice};
use crate::lattices::VarState;
use crate::loaders::utils::VW_Metadata;
use std::default::Default;
use yaxpeax_core::analyses::control_flow::VW_CFG;

use X86Regs::*;

//Top level function
pub fn analyze_jumps(
    cfg: &VW_CFG,
    irmap: &IRMap,
    switch_analyzer: &SwitchAnalyzer,
) -> AnalysisResult<SwitchLattice> {
    run_worklist(cfg, irmap, switch_analyzer)
}

pub struct SwitchAnalyzer {
    pub metadata: VW_Metadata,
    pub reaching_defs: AnalysisResult<ReachLattice>,
    pub reaching_analyzer: ReachingDefnAnalyzer,
}

impl AbstractAnalyzer<SwitchLattice> for SwitchAnalyzer {
    fn aexec_unop(
        &self,
        in_state: &mut SwitchLattice,
        _opcode: &Unopcode,
        dst: &Value,
        src: &Value,
        _loc_idx: &LocIdx,
    ) -> () {
        in_state.set(dst, self.aeval_unop(in_state, src))
    }

    fn aexec_binop(
        &self,
        in_state: &mut SwitchLattice,
        opcode: &Binopcode,
        dst: &Value,
        src1: &Value,
        src2: &Value,
        loc_idx: &LocIdx,
    ) -> () {
        if let Binopcode::Cmp = opcode {
            match (src1, src2) {
                (Value::Reg(regnum, _), Value::Imm(_, _, imm))
                | (Value::Imm(_, _, imm), Value::Reg(regnum, _)) => {
                    let reg_def = self
                        .reaching_analyzer
                        .fetch_def(&self.reaching_defs, loc_idx);
                    let src_loc = reg_def.regs.get_reg(*regnum, ValSize::Size64);
                    in_state.regs.set_reg(Zf, ValSize::Size64,
                        SwitchValueLattice::new(SwitchValue::ZF(*imm as u32, *regnum, src_loc)));
                }
                _ => (),
            }
        }

        match opcode {
            Binopcode::Cmp => (),
            Binopcode::Test => {
                in_state.regs.set_reg(Zf, ValSize::Size64, Default::default());
            }
            _ => in_state.set(dst, self.aeval_binop(in_state, opcode, src1, src2)),
        }
    }

    fn process_branch(
        &self,
        irmap: &IRMap,
        in_state: &SwitchLattice,
        succ_addrs: &Vec<u64>,
        addr: &u64,
    ) -> Vec<(u64, SwitchLattice)> {
        if succ_addrs.len() == 2 {
            let mut not_branch_state = in_state.clone();
            let mut branch_state = in_state.clone();
            if let Some(SwitchValue::ZF(bound, regnum, checked_defs)) = &in_state.regs.get_reg(Zf, ValSize::Size64).v {
                not_branch_state.regs.set_reg(
                    *regnum,
                    ValSize::Size64,
                    SwitchValueLattice {
                        v: Some(SwitchValue::UpperBound(*bound)),
                    },
                );
                let defs_state = self.reaching_defs.get(addr).unwrap();
                let ir_block = irmap.get(addr).unwrap();
                let defs_state = self.reaching_analyzer.analyze_block(defs_state, ir_block);
                //propagate bound across registers with the same reaching def
                for idx in X86Regs::iter() {
                    if idx != *regnum {
                        let reg_def = defs_state.regs.get_reg(idx, ValSize::Size64);
                        if (!reg_def.is_empty()) && (&reg_def == checked_defs) {
                            not_branch_state.regs.set_reg(
                                idx,
                                ValSize::Size64,
                                SwitchValueLattice {
                                    v: Some(SwitchValue::UpperBound(*bound)),
                                },
                            );
                        }
                    }
                }
                //propagate bound across stack slots with the same upper bound
                for (stack_offset, stack_slot) in defs_state.stack.map.iter() {
                    if !checked_defs.is_empty() && (&stack_slot.value == checked_defs) {
                        let v = SwitchValueLattice {
                            v: Some(SwitchValue::UpperBound(*bound)),
                        };
                        let vv = VarSlot {
                            size: stack_slot.size,
                            value: v,
                        };
                        not_branch_state.stack.map.insert(*stack_offset, vv);
                    }
                }
            }
            branch_state.regs.set_reg(Zf, ValSize::Size64, Default::default());
            not_branch_state.regs.set_reg(Zf, ValSize::Size64, Default::default());
            vec![
                (succ_addrs[0].clone(), not_branch_state),
                (succ_addrs[1].clone(), branch_state),
            ]
        } else {
            succ_addrs
                .into_iter()
                .map(|addr| (addr.clone(), in_state.clone()))
                .collect()
        }
    }
}

impl SwitchAnalyzer {
    fn aeval_unop_mem(
        &self,
        in_state: &SwitchLattice,
        memargs: &MemArgs,
        memsize: &ValSize,
    ) -> SwitchValueLattice {
        if let Some(offset) = get_rsp_offset(memargs) {
            return in_state.stack.get(offset, memsize.into_bytes());
        }
        if let MemArgs::MemScale(
            MemArg::Reg(regnum1, size1),
            MemArg::Reg(regnum2, size2),
            MemArg::Imm(_, _, immval),
        ) = memargs
        {
            if let (Some(SwitchValue::SwitchBase(base)), Some(SwitchValue::UpperBound(bound)), 4) = (
                in_state.regs.get_reg(*regnum1, *size1).v,
                in_state.regs.get_reg(*regnum2, *size2).v,
                immval,
            ) {
                return SwitchValueLattice::new(SwitchValue::JmpOffset(base, bound));
            }
        }
        Default::default()
    }

    // 1. if unop is a constant, set as constant -- done
    // 2. if reg, return reg -- done
    // 3. if stack access, return stack access -- done
    // 4. x = mem[switch_base + offset * 4]
    pub fn aeval_unop(&self, in_state: &SwitchLattice, src: &Value) -> SwitchValueLattice {
        match src {
            Value::Mem(memsize, memargs) => self.aeval_unop_mem(in_state, memargs, memsize),
            Value::Reg(regnum, size) => in_state.regs.get_reg(*regnum, *size),
            Value::Imm(_, _, immval) => {
                if *immval == 0 {
                    SwitchValueLattice::new(SwitchValue::UpperBound(1))
                } else {
                    SwitchValueLattice::new(SwitchValue::SwitchBase(*immval as u32))
                }
            }
            Value::RIPConst => Default::default(),
        }
    }

    // 1. x = switch_base + offset
    pub fn aeval_binop(
        &self,
        in_state: &SwitchLattice,
        opcode: &Binopcode,
        src1: &Value,
        src2: &Value,
    ) -> SwitchValueLattice {
        if let Binopcode::Add = opcode {
            if let (Value::Reg(regnum1, size1), Value::Reg(regnum2, size2)) = (src1, src2) {
                match (
                    in_state.regs.get_reg(*regnum1, *size1).v,
                    in_state.regs.get_reg(*regnum2, *size2).v,
                ) {
                    (
                        Some(SwitchValue::SwitchBase(base)),
                        Some(SwitchValue::JmpOffset(_, offset)),
                    )
                    | (
                        Some(SwitchValue::JmpOffset(_, offset)),
                        Some(SwitchValue::SwitchBase(base)),
                    ) => return SwitchValueLattice::new(SwitchValue::JmpTarget(base, offset)),
                    _ => return Default::default(),
                };
            }
        }
        Default::default()
    }
}
