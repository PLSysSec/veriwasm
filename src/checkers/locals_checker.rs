use std::collections::HashSet;
use std::convert::TryFrom;

use crate::analyses::locals_analyzer::LocalsAnalyzer;
use crate::analyses::{AbstractAnalyzer, AnalysisResult};
use crate::checkers::Checker;
use crate::ir::types::{FunType, IRMap, MemArgs, Stmt, Value, VarIndex, X86Regs};
use crate::lattices::localslattice::{LocalsLattice, SlotVal};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::loaders::utils::to_system_v;

use SlotVal::*;
use X86Regs::*;

pub struct LocalsChecker<'a> {
    irmap: &'a IRMap,
    analyzer: &'a LocalsAnalyzer<'a>,
}

pub fn check_locals(
    result: AnalysisResult<LocalsLattice>,
    irmap: &IRMap,
    analyzer: &LocalsAnalyzer,
) -> bool {
    LocalsChecker { irmap, analyzer }.check(result)
}

fn is_uninit_illegal(v: &Value) -> bool {
    match v {
        Value::Mem(memsize, memargs) => true,
        Value::Reg(reg_num, _) => {
            *reg_num != Rsp && *reg_num != Rbp && !(X86Regs::is_flag(*reg_num))
        }
        Value::Imm(_, _, _) => false, //imm are always "init"
        Value::RIPConst => false,
    }
}

impl LocalsChecker<'_> {
    fn all_args_are_init(&self, state: &LocalsLattice, sig: FunType) -> bool {
        for arg in self.analyzer.fun_type.args.iter() {
            match arg {
                (VarIndex::Stack(offset), size) => {
                    let bytesize = size.into_bytes();
                    let v = state.stack.get(i64::from(*offset), bytesize);
                    if v == Uninit {
                        return false;
                    }
                }
                (VarIndex::Reg(reg_num), size) => {
                    let v = state.regs.get_reg(*reg_num, *size);
                    if v == Uninit {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn ret_is_uninitialized(&self, state: &LocalsLattice) -> bool {
        let ret_ty = self.analyzer.fun_type.ret;
        if ret_ty.is_none() {
            false
        } else {
            let (r, sz) = ret_ty.unwrap();
            state.regs.get_reg(r, sz) == Uninit
        }
    }

    // Check if callee-saved registers have been restored properly
    // RSP and RBP are handled by stack analysis
    fn regs_not_restored(&self, state: &LocalsLattice) -> bool {
        for arg in self.analyzer.fun_type.args.iter() {
            match arg {
                (VarIndex::Reg(reg @ R12), size)
                | (VarIndex::Reg(reg @ R13), size)
                | (VarIndex::Reg(reg @ R14), size)
                | (VarIndex::Reg(reg @ R15), size) => {
                    let v = state.regs.get_reg(*reg, *size);
                    if v != InitialRegVal(*reg) {
                        return true;
                    }
                }
                _ => (), // Nothing on the stack is callee-saved
            }
        }
        false
    }
}

impl Checker<LocalsLattice> for LocalsChecker<'_> {
    fn check(&self, result: AnalysisResult<LocalsLattice>) -> bool {
        self.check_state_at_statements(result)
    }

    fn irmap(&self) -> &IRMap {
        self.irmap
    }

    fn aexec(&self, state: &mut LocalsLattice, ir_stmt: &Stmt, loc: &LocIdx) {
        self.analyzer.aexec(state, ir_stmt, loc)
    }

    fn check_statement(&self, state: &LocalsLattice, stmt: &Stmt, loc_idx: &LocIdx) -> bool {
        let debug_addrs: HashSet<u64> = vec![0x00028998].into_iter().collect();
        if debug_addrs.contains(&loc_idx.addr) {
            println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
            println!("{}", state);
            println!("check_statement debug 0x{:x?}: {:?}", loc_idx.addr, stmt);
            let mut cloned = state.clone();
            self.aexec(&mut cloned, stmt, loc_idx);
            println!("{}", cloned);
            println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        }
        let error = match stmt {
            // 1. No writes to registers or memory of uninit values (overly strict, but correct)
            Stmt::Clear(dst, srcs) => {
                (self.analyzer.aeval_vals(state, srcs, loc_idx) == Uninit) && is_uninit_illegal(dst)
            }
            Stmt::Unop(_, dst, src) => {
                (self.analyzer.aeval_val(state, src, loc_idx) == Uninit) && is_uninit_illegal(dst)
            }
            Stmt::Binop(opcode, dst, src1, src2) => {
                self.analyzer
                    .aeval_vals(state, &vec![src1.clone(), src2.clone()], loc_idx)
                    == Uninit
                    && is_uninit_illegal(dst)
            }
            // 2. No branch on uninit allowed
            Stmt::Branch(br_type, val) => self.analyzer.aeval_val(state, val, loc_idx) == Uninit,
            // 3. check that return values are initialized (if the function has any)
            // 3.1 also check that all caller saved regs have been restored
            Stmt::Ret => self.ret_is_uninitialized(state) || self.regs_not_restored(state),
            // 4. TODO: check that all function arguments are initialized (if the called function has any)
            // 4.1 Check direct calls
            Stmt::Call(Value::Imm(_, _, dst)) => {
                let target = (*dst + (loc_idx.addr as i64) + 5) as u64;
                let name = self.analyzer.name_addr_map.get(&target);
                let signature = name
                    .and_then(|name| self.analyzer.symbol_table.indexes.get(name))
                    .and_then(|sig_index| {
                        self.analyzer
                            .symbol_table
                            .signatures
                            .get(*sig_index as usize)
                    });
                if let Some(ty_sig) = signature.map(|sig| to_system_v(sig)) {
                    !self.all_args_are_init(state, ty_sig)
                } else {
                    true
                }
            }
            // 4.2 Check indirect calls
            Stmt::Call(val @ Value::Reg(_, _)) => {
                let fn_ptr_type = self
                    .analyzer
                    .call_analyzer
                    .get_fn_ptr_type(&self.analyzer.call_analysis, loc_idx, val)
                    .and_then(|fn_ptr_index| {
                        self.analyzer
                            .symbol_table
                            .signatures
                            .get(fn_ptr_index as usize)
                    });
                if let Some(ty_sig) = fn_ptr_type.map(|sig| to_system_v(sig)) {
                    !self.all_args_are_init(state, ty_sig)
                } else {
                    true
                }
            }
            _ => false,
        };
        if error {
            println!("----------------------------------------");
            println!("{}", state);
            println!("Darn: 0x{:x?}: {:?}", loc_idx.addr, stmt);
            println!("{:?}", self.analyzer.fun_type);
            println!("----------------------------------------")
        }
        true
    }
}
