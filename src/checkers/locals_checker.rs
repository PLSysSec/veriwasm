use crate::{analyses, ir, lattices, loaders};
use std::collections::HashSet;
use std::convert::TryFrom;

use analyses::locals_analyzer::LocalsAnalyzer;
use analyses::{AbstractAnalyzer, AnalysisResult};
use checkers::Checker;
use ir::types::{FunType, IRMap, MemArgs, Stmt, ValSize, Value, VarIndex, X86Regs};
use ir::utils::is_stack_access;
use lattices::localslattice::{LocalsLattice, SlotVal};
use lattices::reachingdefslattice::LocIdx;
use loaders::utils::is_libcall;
use loaders::utils::to_system_v;
use yaxpeax_x86::long_mode::Opcode;

use SlotVal::*;
use ValSize::*;
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

fn is_noninit_illegal(v: &Value) -> bool {
    match v {
        Value::Mem(memsize, memargs) => !is_stack_access(v),
        Value::Reg(reg_num, _) => false,
        // {
        //     *reg_num != Rsp && *reg_num != Rbp && !(X86Regs::is_flag(*reg_num))
        // },
        // false,
        Value::Imm(_, _, _) => false, //imm are always "init"
        Value::RIPConst => false,
    }
}

impl LocalsChecker<'_> {
    fn all_args_are_init(&self, state: &LocalsLattice, sig: FunType) -> bool {
        for arg in sig.args.iter() {
            match arg {
                (VarIndex::Stack(offset), size) => {
                    let bytesize = size.into_bytes();
                    // -8 is because the return address has not been pushed
                    let v = state.stack.get(i64::from(*offset - 8), bytesize);
                    if v != Init {
                        println!(
                            "found arg that was not initialized: stack[{:?}] sig: {:?}",
                            offset - 8,
                            sig
                        );
                        return false;
                    }
                }
                (VarIndex::Reg(reg), size) => {
                    let v = state.regs.get_reg(*reg, *size);
                    if v != Init {
                        println!(
                            "found arg that was not initialized: {:?} sig: {:?}",
                            reg, sig
                        );
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
            state.regs.get_reg(r, sz) != Init
        }
    }

    // Check if callee-saved registers have been restored properly
    // RSP and RBP are handled by stack analysis
    fn regs_not_restored(&self, state: &LocalsLattice) -> bool {
        for reg in vec![Rbx, R12, R13, R14, R15].iter() {
            let v = state.regs.get_reg(*reg, Size64);
            if v != UninitCalleeReg(*reg) {
                return true;
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
        let debug_addrs: HashSet<u64> = vec![].into_iter().collect();
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
            // 1. No writes to memory of uninit values
            Stmt::Clear(dst, srcs) => {
                (self.analyzer.aeval_vals(state, srcs, loc_idx) != Init) && is_noninit_illegal(dst)
            }
            Stmt::Unop(_, dst, src) => {
                (self.analyzer.aeval_val(state, src, loc_idx) != Init) && is_noninit_illegal(dst)
            }
            Stmt::Binop(opcode, dst, src1, src2) => {
                self.analyzer
                    .aeval_vals(state, &vec![src1.clone(), src2.clone()], loc_idx)
                    != Init
                    && is_noninit_illegal(dst)
            }
            // 2. No branch on uninit allowed
            Stmt::Branch(br_type, val) => match br_type {
                Opcode::JO | Opcode::JNO => state.regs.get_reg(Of, Size8) != Init,
                Opcode::JB | Opcode::JNB => state.regs.get_reg(Cf, Size8) != Init,
                Opcode::JZ | Opcode::JNZ => state.regs.get_reg(Zf, Size8) != Init,
                Opcode::JA | Opcode::JNA => {
                    state.regs.get_reg(Cf, Size8) != Init || state.regs.get_reg(Zf, Size8) != Init
                }
                Opcode::JS | Opcode::JNS => state.regs.get_reg(Sf, Size8) != Init,
                Opcode::JP | Opcode::JNP => state.regs.get_reg(Pf, Size8) != Init,
                Opcode::JL | Opcode::JGE => {
                    state.regs.get_reg(Sf, Size8) != Init || state.regs.get_reg(Of, Size8) != Init
                }
                Opcode::JG | Opcode::JLE => {
                    state.regs.get_reg(Sf, Size8) != Init
                        || state.regs.get_reg(Sf, Size8) != Init
                        || state.regs.get_reg(Of, Size8) != Init
                }
                _ => false,
            },
            // self.analyzer.aeval_val(state, val, loc_idx) != Init,
            // 3. check that return values are initialized (if the function has any)
            // 3.1 also check that all caller saved regs have been restored
            Stmt::Ret => self.ret_is_uninitialized(state) || self.regs_not_restored(state),
            // 4. check that all function arguments are initialized (if the called function has any)
            Stmt::Call(val) => {
                let signature = match val {
                    // 4.1 Check direct calls
                    Value::Imm(_, _, dst) => {
                        let target = (*dst + (loc_idx.addr as i64) + 5) as u64;
                        let name = self.analyzer.name_addr_map.get(&target);
                        let v = name
                            .and_then(|name| self.analyzer.symbol_table.indexes.get(name))
                            .and_then(|sig_index| {
                                self.analyzer
                                    .symbol_table
                                    .signatures
                                    .get(*sig_index as usize)
                            });
                        if let Some(n) = name {
                            if is_libcall(n) {
                                return true;
                            }
                        }
                        v
                    }
                    // 4.2 Check indirect calls
                    Value::Reg(_, _) => self
                        .analyzer
                        .call_analyzer
                        .get_fn_ptr_type(&self.analyzer.call_analysis, loc_idx, val)
                        .and_then(|fn_ptr_index| {
                            self.analyzer
                                .symbol_table
                                .signatures
                                .get(fn_ptr_index as usize)
                        }),
                    _ => panic!("bad call value: {:?}", val),
                };
                let type_check_result = if let Some(ty_sig) = signature.map(|sig| to_system_v(sig))
                {
                    !self.all_args_are_init(state, ty_sig)
                } else {
                    true
                };
                // checks that call targets aren't uninitialized values
                type_check_result || self.analyzer.aeval_val(state, val, loc_idx) != Init
            }
            _ => false,
        };
        if error {
            println!("----------------------------------------");
            println!("{}", state);
            println!("Darn: 0x{:x?}: {:?}", loc_idx.addr, stmt);
            // println!("", self.irmap.get(loc_idx));
            // println!("{:?}", self.analyzer.fun_type);
            println!("----------------------------------------")
        }
        !error
    }
}
