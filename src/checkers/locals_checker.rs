use std::collections::HashSet;
use std::convert::TryFrom;

use crate::analyses::locals_analyzer::LocalsAnalyzer;
use crate::analyses::{AbstractAnalyzer, AnalysisResult};
use crate::checkers::Checker;
use crate::ir::types::{IRMap, MemArgs, Stmt, Value, X86Regs};
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
                self.analyzer.aeval_vals(state, srcs, loc_idx) == &&is_uninit_illegal(dst)
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
