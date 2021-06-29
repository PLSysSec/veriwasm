use std::collections::HashMap;

use crate::ir::types::FunType;
use crate::lattices::calllattice::CallCheckLattice;
use crate::analyses::{AbstractAnalyzer, AnalysisResult};
use crate::loaders::utils::{VwFuncInfo, to_system_v_ret_ty};
use crate::lattices::mem_to_stack_offset;
use crate::ir::types::{Value, Binopcode, Stmt, IRMap, ValSize};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::localslattice::*;
use crate::lattices::{VariableState, VarState, Lattice, VarIndex};

use SlotVal::*;
use X86Regs::*;

pub struct LocalsAnalyzer<'a> {
    pub fun_type: FunType,
    pub symbol_table: &'a VwFuncInfo,
    pub name_addr_map: &'a HashMap<u64, String>,
    pub plt_bounds: (u64, u64),
    pub call_analysis: AnalysisResult<CallCheckLattice>,
}

impl<'a> LocalsAnalyzer<'a> {
    pub fn aeval_val(&self, state: &LocalsLattice, value: &Value) -> SlotVal {
        match value {
            Value::Mem(memsize, memargs) => {
                if let Some(offset) = mem_to_stack_offset(memargs) {
                    state.stack.get(offset, memsize.into_bytes())
                } else {
                    Init
                }
            },
            Value::Reg(_, _) => state.get(value).unwrap_or(Uninit),
            Value::Imm(_, _, _) => Init,
            Value::RIPConst => todo!(),
        }
    }

    // if all values are initialized then the value is initialized
    pub fn aeval_vals(&self, state: &LocalsLattice, values: &Vec<Value>) -> SlotVal {
        values.iter().fold(Init, |acc, value| -> SlotVal {
            if (acc == Init) && (self.aeval_val(state, value) == Init) {
                Init
            } else {
                Uninit
            }})
    }
}

impl<'a> AbstractAnalyzer<LocalsLattice> for LocalsAnalyzer<'a> {
    fn init_state(&self) -> LocalsLattice {
        let mut lattice: LocalsLattice = Default::default();
        for arg in self.fun_type.args.iter() {
            match arg {
                (VarIndex::Stack(offset), size) => {
                    lattice.stack.update(i64::from(*offset), Init, size.into_bytes())
                },
                (VarIndex::Reg(reg_num), size) => {
                    lattice.regs.set_reg(*reg_num, *size, Init)
                },
            }
        }
        // rbp, rbx, and r12-r15 are the callee-saved registers
        lattice.regs.set_reg(Rbp, ValSize::Size64, InitialRegVal(Rbp));
        lattice.regs.set_reg(Rbx, ValSize::Size64, InitialRegVal(Rbx));
        lattice.regs.set_reg(R12, ValSize::Size64, InitialRegVal(R12));
        lattice.regs.set_reg(R13, ValSize::Size64, InitialRegVal(R13));
        lattice.regs.set_reg(R14, ValSize::Size64, InitialRegVal(R14));
        lattice.regs.set_reg(R15, ValSize::Size64, InitialRegVal(R15));
        lattice
    }

    fn aexec(&self, in_state: &mut LocalsLattice, ir_instr: &Stmt, loc_idx: &LocIdx) -> () {
        // if loc_idx.addr == 0xf5e5 {
        //     println!("IN STATE 0x{:x?}: {:?}\n{:?}", loc_idx.addr, ir_instr, in_state);
        // }
        match ir_instr {
            Stmt::Clear(dst, srcs) => {
                in_state.set(dst, self.aeval_vals(in_state, srcs))
            }
            Stmt::Unop(_, dst, src) => {
                in_state.set(dst, self.aeval_val(in_state, src))
            }
            Stmt::Binop(opcode, dst, src1, src2) => {
                let dst_val = self.aeval_val(in_state, src1).meet(&self.aeval_val(in_state, src2), loc_idx);
                in_state.set(dst, dst_val);
                in_state.adjust_stack_offset(opcode, dst, src1, src2);
            }
            // TODO: wasi calls (no requirements on initialization, always return in rax)
            // TODO: wasi calls are things in plt_bounds?
            Stmt::Call(Value::Imm(_, _, dst)) => {
                let target = (*dst + (loc_idx.addr as i64) + 5) as u64;
                if self.plt_bounds.0 <= target && target <= self.plt_bounds.1 { // plt (TODO: trusted?) function
                    in_state.on_call(); // clear caller-saved registers
                    in_state.regs.set_reg(Rax, ValSize::Size64, Init); // TODO: assumes that all calls in plt return in rax
                } else { // library function
                    let name = self.name_addr_map.get(&target);
                    let signature = name
                        .and_then(|name| self.symbol_table.indexes.get(name))
                        .and_then(|sig_index| self.symbol_table.signatures.get(*sig_index as usize));
                    in_state.on_call();
                    if let Some((ret_reg, reg_size)) = signature.and_then(|sig| to_system_v_ret_ty(sig)) {
                        in_state.regs.set_reg(ret_reg, reg_size, Init);
                    }
                }
            },
            Stmt::Call(Value::Reg(reg_num, reg_size)) => {
                // println!("INDIRECT: 0x{:x?}: {:?}", loc_idx.addr, self.call_analysis.get(&loc_idx.addr));
                // panic!("0x{:x?}: {:?}", loc_idx.addr, dst);
                // println!("indirect call 0x{:x?}", loc_idx.addr);
            },
            Stmt::Branch(br_type, val) => {
                // println!("unhandled branch 0x{:x?}: {:?} {:?}", loc_idx.addr, br_type, val);
            },
            stmt => {
                // println!("unhandled instruction 0x{:x?}: {:?}", loc_idx.addr, stmt);
            }
        }
        // if loc_idx.addr == 0xf5e5 {
        //     println!("OUT STATE\n{:?}", in_state);
        // }
    }

    fn process_branch(
        &self,
        _irmap: &IRMap,
        in_state: &LocalsLattice,
        succ_addrs: &Vec<u64>,
        _addr: &u64,
    ) -> Vec<(u64, LocalsLattice)> {
        succ_addrs
            .into_iter()
            .map(|addr| (addr.clone(), in_state.clone()))
            .collect()
    }
}
