use crate::{analyses, ir, lattices, loaders};
use analyses::reaching_defs::ReachingDefnAnalyzer;
use analyses::{AbstractAnalyzer, AnalysisResult};
use ir::types::{Binopcode, IRMap, MemArg, MemArgs, Unopcode, ValSize, Value, X86Regs, RegT};
use ir::utils::get_rsp_offset;
use lattices::reachingdefslattice::{LocIdx, ReachLattice};
use lattices::switchlattice::{SwitchLattice, SwitchValue, SwitchValueLattice};
use lattices::{VarSlot, VarState};
use loaders::types::VwMetadata;
use std::default::Default;

use SwitchValue::{JmpOffset, JmpTarget, SwitchBase, UpperBound};
use ValSize::*;
use X86Regs::*;

pub struct SwitchAnalyzer<Ar: RegT> {
    pub metadata: VwMetadata,
    pub reaching_defs: AnalysisResult<ReachLattice<Ar>>,
    pub reaching_analyzer: ReachingDefnAnalyzer<Ar>,
}

impl<Ar: RegT> AbstractAnalyzer<Ar, SwitchLattice<Ar>> for SwitchAnalyzer<Ar> {
    fn aexec_unop(
        &self,
        in_state: &mut SwitchLattice<Ar>,
        _opcode: &Unopcode,
        dst: &Value<Ar>,
        src: &Value<Ar>,
        _loc_idx: &LocIdx,
    ) -> () {
        in_state.set(dst, self.aeval_unop(in_state, src))
    }

    fn aexec_binop(
        &self,
        in_state: &mut SwitchLattice<Ar>,
        opcode: &Binopcode,
        dst: &Value<Ar>,
        src1: &Value<Ar>,
        src2: &Value<Ar>,
        loc_idx: &LocIdx,
    ) -> () {
        if let Binopcode::Cmp = opcode {
            match (src1, src2) {
                (Value::Reg(regnum, _), Value::Imm(_, _, imm))
                | (Value::Imm(_, _, imm), Value::Reg(regnum, _)) => {
                    let reg_def = self
                        .reaching_analyzer
                        .fetch_def(&self.reaching_defs, loc_idx);
                    let src_loc = reg_def.regs.get_reg(*regnum, Size64);
                    in_state.regs.set_reg(
                        Zf,
                        Size64,
                        SwitchValueLattice::new(SwitchValue::ZF(*imm as u32, *regnum, src_loc)),
                    );
                }
                _ => (),
            }
        }

        match opcode {
            Binopcode::Cmp => (),
            Binopcode::Test => {
                in_state.regs.set_reg(Zf, Size64, Default::default());
            }
            _ => in_state.set(dst, self.aeval_binop(in_state, opcode, src1, src2)),
        }
    }

    fn process_branch(
        &self,
        irmap: &IRMap<Ar>,
        in_state: &SwitchLattice<Ar>,
        succ_addrs: &Vec<u64>,
        addr: &u64,
    ) -> Vec<(u64, SwitchLattice<Ar>)> {
        if succ_addrs.len() == 2 {
            let mut not_branch_state = in_state.clone();
            let mut branch_state = in_state.clone();
            if let Some(SwitchValue::ZF(bound, regnum, checked_defs)) =
                &in_state.regs.get_reg(Zf, Size64).v
            {
                not_branch_state.regs.set_reg(
                    *regnum,
                    Size64,
                    SwitchValueLattice {
                        v: Some(UpperBound(*bound)),
                    },
                );
                let defs_state = self.reaching_defs.get(addr).unwrap();
                let ir_block = irmap.get(addr).unwrap();
                let defs_state = self.reaching_analyzer.analyze_block(defs_state, ir_block);
                //propagate bound across registers with the same reaching def
                for idx in X86Regs::iter() {
                    if idx != *regnum {
                        let reg_def = defs_state.regs.get_reg(idx, Size64);
                        if (!reg_def.is_empty()) && (&reg_def == checked_defs) {
                            not_branch_state.regs.set_reg(
                                idx,
                                Size64,
                                SwitchValueLattice {
                                    v: Some(UpperBound(*bound)),
                                },
                            );
                        }
                    }
                }
                //propagate bound across stack slots with the same upper bound
                for (stack_offset, stack_slot) in defs_state.stack.map.iter() {
                    if !checked_defs.is_empty() && (&stack_slot.value == checked_defs) {
                        let v = SwitchValueLattice {
                            v: Some(UpperBound(*bound)),
                        };
                        let vv = VarSlot {
                            size: stack_slot.size,
                            value: v,
                        };
                        not_branch_state.stack.map.insert(*stack_offset, vv);
                    }
                }
            }
            branch_state.regs.set_reg(Zf, Size64, Default::default());
            not_branch_state
                .regs
                .set_reg(Zf, Size64, Default::default());
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

impl<Ar: RegT> SwitchAnalyzer<Ar> {
    fn aeval_unop_mem(
        &self,
        in_state: &SwitchLattice<Ar>,
        memargs: &MemArgs<Ar>,
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
            if let (Some(SwitchBase(base)), Some(UpperBound(bound)), 4) = (
                in_state.regs.get_reg(*regnum1, *size1).v,
                in_state.regs.get_reg(*regnum2, *size2).v,
                immval,
            ) {
                return SwitchValueLattice::new(JmpOffset(base, bound));
            }
        }
        Default::default()
    }

    // 1. if unop is a constant, set as constant -- done
    // 2. if reg, return reg -- done
    // 3. if stack access, return stack access -- done
    // 4. x = mem[switch_base + offset * 4]
    pub fn aeval_unop(&self, in_state: &SwitchLattice<Ar>, src: &Value<Ar>) -> SwitchValueLattice {
        match src {
            Value::Mem(memsize, memargs) => self.aeval_unop_mem(in_state, memargs, memsize),
            Value::Reg(regnum, size) => in_state.regs.get_reg(*regnum, *size),
            Value::Imm(_, _, immval) => {
                if *immval == 0 {
                    SwitchValueLattice::new(UpperBound(1))
                } else {
                    SwitchValueLattice::new(SwitchBase(*immval as u32))
                }
            }
            Value::RIPConst => Default::default(),
        }
    }

    // 1. x = switch_base + offset
    pub fn aeval_binop(
        &self,
        in_state: &SwitchLattice<Ar>,
        opcode: &Binopcode,
        src1: &Value<Ar>,
        src2: &Value<Ar>,
    ) -> SwitchValueLattice {
        if let Binopcode::Add = opcode {
            if let (Value::Reg(regnum1, size1), Value::Reg(regnum2, size2)) = (src1, src2) {
                match (
                    in_state.regs.get_reg(*regnum1, *size1).v,
                    in_state.regs.get_reg(*regnum2, *size2).v,
                ) {
                    (Some(SwitchBase(base)), Some(JmpOffset(_, offset)))
                    | (Some(JmpOffset(_, offset)), Some(SwitchBase(base))) => {
                        return SwitchValueLattice::new(JmpTarget(base, offset))
                    }
                    _ => return Default::default(),
                };
            }
        }
        Default::default()
    }
}
