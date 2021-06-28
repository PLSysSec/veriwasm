use crate::analyses::reaching_defs::ReachingDefnAnalyzer;
use crate::analyses::AbstractAnalyzer;
use crate::analyses::AnalysisResult;
use crate::ir::types::{
    Binopcode, IRBlock, IRMap, MemArg, MemArgs, Stmt, Unopcode, ValSize, Value,
};
use crate::ir::utils::{extract_stack_offset, is_stack_access};
use crate::lattices::calllattice::{CallCheckLattice, CallCheckValue, CallCheckValueLattice};
use crate::lattices::davlattice::DAV;
use crate::lattices::reachingdefslattice::{LocIdx, ReachLattice};
use crate::lattices::VarSlot;
use crate::lattices::X86Regs::*;
use crate::lattices::VarState;
use crate::lattices::X86Regs;
use crate::loaders::utils::VW_Metadata;
use std::default::Default;
use yaxpeax_x86::long_mode::Opcode;
use std::convert::TryFrom;
use yaxpeax_core::analyses::control_flow::VW_CFG;


pub struct CallAnalyzer {
    pub metadata: VW_Metadata,
    pub reaching_defs: AnalysisResult<ReachLattice>,
    pub reaching_analyzer: ReachingDefnAnalyzer,
    pub funcs: Vec<u64>,
    pub irmap: IRMap,
    pub cfg: VW_CFG,
}


impl CallAnalyzer {
    //1. get enclosing block addr
    //2. get result for that block start
    //3. get result for specific addr
    pub fn fetch_result(
        &self,
        result: &AnalysisResult<CallCheckLattice>,
        loc_idx: &LocIdx,
    ) -> CallCheckLattice {
        if self.cfg.blocks.contains_key(&loc_idx.addr) {
            return result.get(&loc_idx.addr).unwrap().clone();
        }
        let block_addr = self.cfg.prev_block(loc_idx.addr).unwrap().start;
        let irblock = self.irmap.get(&block_addr).unwrap();
        let mut a_state = result.get(&block_addr).unwrap().clone();
        for (addr, instruction) in irblock.iter() {
            for (idx, ir_insn) in instruction.iter().enumerate() {
                if &loc_idx.addr == addr && (loc_idx.idx as usize) == idx {
                    return a_state;
                }
                self.aexec(
                    &mut a_state,
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



impl AbstractAnalyzer<CallCheckLattice> for CallAnalyzer {
    fn analyze_block(&self, state: &CallCheckLattice, irblock: &IRBlock) -> CallCheckLattice {
        let mut new_state = state.clone();
        for (addr, instruction) in irblock.iter() {
            for (idx, ir_insn) in instruction.iter().enumerate() {
                log::debug!(
                    "Call analyzer: stmt at 0x{:x}: {:?} with state {:?}",
                    addr,
                    ir_insn,
                    new_state
                );
                self.aexec(
                    &mut new_state,
                    ir_insn,
                    &LocIdx {
                        addr: *addr,
                        idx: idx as u32,
                    },
                );
            }
        }
        new_state
    }

    fn aexec_unop(
        &self,
        in_state: &mut CallCheckLattice,
        _opcode: &Unopcode,
        dst: &Value,
        src: &Value,
        _loc_idx: &LocIdx,
    ) -> () {
        in_state.set(dst, self.aeval_unop(in_state, src))
    }

    fn aexec_binop(
        &self,
        in_state: &mut CallCheckLattice,
        opcode: &Binopcode,
        dst: &Value,
        src1: &Value,
        src2: &Value,
        loc_idx: &LocIdx,
    ) -> () {
        match (opcode,src1,src2) {
            (Binopcode::Cmp, Value::Reg(regnum1, size1), Value::Reg(regnum2, size2)) => {
                if let Some(CallCheckValue::TableSize) = in_state.regs.get_reg_index(*regnum2, *size2).v {
                    in_state.regs.set_reg(Zf, ValSize::Size64,
                        CallCheckValueLattice::new(CallCheckValue::CheckFlag(0, *regnum1)))
                    }
                if let Some(CallCheckValue::TableSize) = in_state.regs.get_reg_index(*regnum1, *size1).v {
                    in_state.regs.set_reg(Zf, ValSize::Size64,
                        CallCheckValueLattice::new(CallCheckValue::CheckFlag(0, *regnum2)))
                    }
                }
            (Binopcode::Cmp, Value::Reg(regnum, size), Value::Imm(_,_,c)) => {
                if let Some(CallCheckValue::TypeOf(r)) = in_state.regs.get_reg_index(*regnum, *size).v {
                    log::debug!("{:x}: Settng zf = TypeCheckFlag({:?}, {:?}) from {:?}", loc_idx.addr,X86Regs::try_from(r).unwrap(), c, X86Regs::try_from(*regnum).unwrap());
                    in_state.regs.set_reg(Zf, ValSize::Size64,
                        CallCheckValueLattice::new(CallCheckValue::TypeCheckFlag(r, *c as u32)))
                    }
            }
            (Binopcode::Test,_,_) => (),
            _ => in_state.set(dst, self.aeval_binop(in_state, opcode, src1, src2, loc_idx)),
            } 
        }

    fn process_branch(
        &self,
        irmap: &IRMap,
        in_state: &CallCheckLattice,
        succ_addrs: &Vec<u64>,
        addr: &u64,
    ) -> Vec<(u64, CallCheckLattice)> {
        let br_stmt = irmap
            .get(addr)
            .expect("no instruction at given address")
            .last()
            .expect("no instructions in block")
            .1
            .last()
            .expect("no IR instructions for last disassembled instruction");
        let br_opcode = match br_stmt {
            Stmt::Branch(op, _) => Some(op),
            _ => None,
        };
        let (is_unsigned_cmp, is_je, flip) = match br_opcode {
            Some(Opcode::JB) => (true, false, false),
            Some(Opcode::JNB) => (true, false, true),
            Some(Opcode::JZ) => (false, true, false),
            _ => (false, false, false),
        };

        log::debug!("{:x}: is_unsigned_cmp {} is_je {} flip {}", addr, is_unsigned_cmp, is_je, flip);

        if succ_addrs.len() == 2 {
            let mut not_branch_state = in_state.clone();
            let mut branch_state = in_state.clone();
            if is_unsigned_cmp{
                if let Some(CallCheckValue::CheckFlag(_, regnum)) = not_branch_state.regs.get_reg(Zf, ValSize::Size64).v {
                    log::debug!("branch at 0x{:x}: CheckFlag for reg {}", addr, regnum);
                    let new_val = CallCheckValueLattice {
                        v: Some(CallCheckValue::CheckedVal),
                    };
                    branch_state
                        .regs
                        .set_reg_index(&regnum, &ValSize::Size64, new_val.clone());
                    //1. propagate checked values
                    let defs_state = self.reaching_defs.get(addr).unwrap();
                    let ir_block = irmap.get(addr).unwrap();
                    let defs_state = self.reaching_analyzer.analyze_block(defs_state, ir_block);
                    let checked_defs = defs_state.regs.get_reg_index(regnum, ValSize::Size64);
                    for idx in 0..15 {
                        let reg_def = defs_state.regs.get_reg_index(idx, ValSize::Size64);
                        if (!reg_def.is_empty()) && (reg_def == checked_defs) {
                            branch_state
                                .regs
                                .set_reg_index(&idx, &ValSize::Size64, new_val.clone());
                        }
                    }

                    for (stack_offset, stack_slot) in defs_state.stack.map.iter() {
                        if !checked_defs.is_empty() && (stack_slot.value == checked_defs) {
                            let vv = VarSlot {
                                size: stack_slot.size,
                                value: new_val.clone(),
                            };
                            branch_state.stack.map.insert(*stack_offset, vv);
                        }
                    }

                    //3. resolve ptr thunks in registers
                    let checked_ptr = CallCheckValueLattice {
                        v: Some(CallCheckValue::PtrOffset(DAV::Checked)),
                    };
                    for idx in 0..15 {
                        let reg_val = branch_state.regs.get_reg_index(idx, ValSize::Size64);
                        if let Some(CallCheckValue::PtrOffset(DAV::Unchecked(reg_def))) = reg_val.v {
                            if checked_defs.is_empty() && reg_def == checked_defs {
                                branch_state
                                    .regs
                                    .set_reg_index(&idx, &ValSize::Size64, checked_ptr.clone());
                            }
                        }
                    }

                    //4. resolve ptr thunks in stack slots --
                    for (stack_offset, stack_slot) in not_branch_state.stack.map.iter() {
                        let stack_val = stack_slot.value.v.clone();
                        if let Some(CallCheckValue::PtrOffset(DAV::Unchecked(stack_def))) = stack_val {
                            if !checked_defs.is_empty() && (stack_def == checked_defs) {
                                let v = VarSlot {
                                    size: stack_slot.size,
                                    value: checked_ptr.clone(),
                                };
                                branch_state.stack.map.insert(*stack_offset, v);
                            }
                        }
                    }
                }
            }
            else if is_je{
                //Handle TypeCheck
                if let Some(CallCheckValue::TypeCheckFlag(regnum, c)) = not_branch_state.regs.get_reg(Zf, ValSize::Size64).v {
                    log::debug!("branch at 0x{:x}: TypeCheckFlag for reg {}. Making TypedPtrOffset({:?})", addr, regnum, c);
                    let new_val = CallCheckValueLattice {
                        v: Some(CallCheckValue::TypedPtrOffset(c)),
                    };
                    branch_state
                        .regs
                        .set_reg_index(&regnum, &ValSize::Size64, new_val.clone());
                }
            }

            branch_state.regs.set_reg(Zf, ValSize::Size64, Default::default());
            not_branch_state.regs.set_reg(Zf, ValSize::Size64, Default::default());

            log::debug!(
                " ->     branch_state @ 0x{:x} = {:?}",
                succ_addrs[1],
                branch_state
            );
            log::debug!(
                " -> not_branch_state @ 0x{:x} = {:?}",
                succ_addrs[0],
                not_branch_state
            );

            if flip {
                vec![
                    (succ_addrs[0].clone(), branch_state),
                    (succ_addrs[1].clone(), not_branch_state),
                ]
            } else {
                vec![
                    (succ_addrs[0].clone(), not_branch_state),
                    (succ_addrs[1].clone(), branch_state),
                ]
            }
        } else {
            succ_addrs
                .into_iter()
                .map(|addr| (addr.clone(), in_state.clone()))
                .collect()
        }
    }
}

// mem[LucetTableBase + 8]
pub fn is_table_size(in_state: &CallCheckLattice, memargs: &MemArgs) -> bool {
    if let MemArgs::Mem2Args(MemArg::Reg(regnum1, size), MemArg::Imm(_, _, 8)) = memargs {
        if let Some(CallCheckValue::LucetTablesBase) = in_state.regs.get_reg_index(*regnum1, *size).v {
            return true;
        }
    }
    false
}

pub fn is_fn_ptr(in_state: &CallCheckLattice, memargs: &MemArgs) -> Option<u32> {
    if let MemArgs::Mem3Args(
        MemArg::Reg(regnum1, size1),
        MemArg::Reg(regnum2, size2),
        MemArg::Imm(_, _, immval),
    ) = memargs
    {
        match (
            in_state.regs.get_reg_index(*regnum1, *size1).v,
            in_state.regs.get_reg_index(*regnum2, *size2).v,
            immval,
        ) {
            (
                Some(CallCheckValue::GuestTableBase),
                Some(CallCheckValue::TypedPtrOffset(c)),
                // Some(CallCheckValue::PtrOffset(DAV::Checked)),
                8,
            ) => return Some(c),
            (
                Some(CallCheckValue::TypedPtrOffset(c)),
                Some(CallCheckValue::GuestTableBase),
                8,
            ) => return Some(c),
            _ => return None,
        }
    }
    None
}

// returns none if not a typeof op
// returns Some(regnum) if result is type of regnum
pub fn is_typeof(in_state: &CallCheckLattice, memargs: &MemArgs) -> Option<u8> {
    if let MemArgs::Mem2Args(
        MemArg::Reg(regnum1, size1),
        MemArg::Reg(regnum2, size2),
    ) = memargs
    {
        match (
            in_state.regs.get_reg_index(*regnum1, *size1).v,
            in_state.regs.get_reg_index(*regnum2, *size2).v,
        ) {
            (
                Some(CallCheckValue::GuestTableBase),
                Some(CallCheckValue::PtrOffset(DAV::Checked)),
            ) => return Some(*regnum2),
            (
                Some(CallCheckValue::PtrOffset(DAV::Checked)),
                Some(CallCheckValue::GuestTableBase),
            ) => return Some(*regnum1),
            _ => return None,
        }
    }
    None
}

impl CallAnalyzer {
    fn is_func_start(&self, addr: u64) -> bool {
        self.funcs.contains(&addr)
    }

    pub fn aeval_unop(&self, in_state: &CallCheckLattice, value: &Value) -> CallCheckValueLattice {
        match value {
            Value::Mem(memsize, memargs) => {
                if is_table_size(in_state, memargs) {
                    return CallCheckValueLattice {
                        v: Some(CallCheckValue::TableSize),
                    };
                } else if is_fn_ptr(in_state, memargs).is_some() {
                    let ty = is_fn_ptr(in_state, memargs).unwrap();
                    log::debug!("Making FnPtr({:?})", ty);
                    return CallCheckValueLattice {
                        v: Some(CallCheckValue::FnPtr(ty)),
                    };
                } else if is_typeof(in_state, memargs).is_some() {
                    let reg = is_typeof(in_state, memargs).unwrap();
                    log::debug!("Typeof operation: args: {:?} => r{:?} is base",  memargs, reg);
                    return CallCheckValueLattice {
                        v: Some(CallCheckValue::TypeOf(reg)),
                    };
                } else if is_stack_access(value) {
                    let offset = extract_stack_offset(memargs);
                    return in_state.stack.get(offset, memsize.into_bytes());
                }
            }

            Value::Reg(regnum, size) => return in_state.regs.get_reg_index(*regnum, *size),

            Value::Imm(_, _, immval) => {
                if (*immval as u64) == self.metadata.guest_table_0 {
                    return CallCheckValueLattice {
                        v: Some(CallCheckValue::GuestTableBase),
                    };
                } else if (*immval as u64) == self.metadata.lucet_tables {
                    return CallCheckValueLattice {
                        v: Some(CallCheckValue::LucetTablesBase),
                    };
                } else if self.is_func_start(*immval as u64) {
                    return CallCheckValueLattice {
                        v: Some(CallCheckValue::FnPtr(1337)),//dummy value, TODO remove
                    };
                }
            }

            Value::RIPConst => {
                // The backend uses rip-relative data to embed constant function pointers.
                return CallCheckValueLattice {
                    v: Some(CallCheckValue::FnPtr(1337)),//Dummy value, TODO remove
                };
            }
        }
        Default::default()
    }

    //checked_val << 4
    pub fn aeval_binop(
        &self,
        in_state: &CallCheckLattice,
        opcode: &Binopcode,
        src1: &Value,
        src2: &Value,
        loc_idx: &LocIdx,
    ) -> CallCheckValueLattice {
        if let Binopcode::Shl = opcode {
            if let (Value::Reg(regnum1, size1), Value::Imm(_, _, 4)) = (src1, src2) {
                if let Some(CallCheckValue::CheckedVal) = in_state.regs.get_reg_index(*regnum1, *size1).v {
                    return CallCheckValueLattice {
                        v: Some(CallCheckValue::PtrOffset(DAV::Checked)),
                    };
                } else {
                    let def_state = self
                        .reaching_analyzer
                        .fetch_def(&self.reaching_defs, loc_idx);
                    let reg_def = def_state.regs.get_reg_index(*regnum1, *size1);
                    return CallCheckValueLattice {
                        v: Some(CallCheckValue::PtrOffset(DAV::Unchecked(reg_def))),
                    };
                }
            }
        }
        Default::default()
    }
}
