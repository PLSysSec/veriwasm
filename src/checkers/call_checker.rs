use yaxpeax_core::arch::x86_64::x86_64Data;
use crate::analyses::call_analyzer::CallAnalyzer;
use crate::analyses::{AbstractAnalyzer, AnalysisResult};
use crate::checkers::Checker;
use crate::lattices::calllattice::{CallCheckLattice, CallCheckValue};
use crate::lattices::davlattice::DAV;
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lifter::{IRMap, MemArg, MemArgs, Stmt, ValSize, Value};

pub struct CallChecker<'a> {
    irmap: &'a IRMap,
    analyzer: &'a CallAnalyzer,
    // funcs: &'a Vec<u64>,
    // x86_64_data: &'a x86_64Data,
}

pub fn check_calls(
    result: AnalysisResult<CallCheckLattice>,
    irmap: &IRMap,
    analyzer: &CallAnalyzer,
    // funcs: &Vec<u64>,
    // x86_64_data: &x86_64Data,
) -> bool {
    CallChecker {
        irmap,
        analyzer,
        // funcs,
        // x86_64_data,
    }
    .check(result)
}

impl Checker<CallCheckLattice> for CallChecker<'_> {
    fn check(&self, result: AnalysisResult<CallCheckLattice>) -> bool {
        self.check_state_at_statements(result)
    }

    fn irmap(&self) -> &IRMap {
        self.irmap
    }
    fn aexec(&self, state: &mut CallCheckLattice, ir_stmt: &Stmt, loc: &LocIdx) {
        self.analyzer.aexec(state, ir_stmt, loc)
    }

    fn check_statement(&self, state: &CallCheckLattice, ir_stmt: &Stmt, loc_idx: &LocIdx) -> bool {
        //1. Check that all indirect calls use resolved function pointer
        if let Stmt::Call(v) = ir_stmt {
            if !self.check_indirect_call(state, v, loc_idx) {
                println!("Failure Case: Indirect Call");
                return false;
            }
        }

        // 2. Check that lookup is using resolved DAV
        if let Stmt::Unop(_, _, Value::Mem(_, memargs)) = ir_stmt {
            if !self.check_calltable_lookup(state, memargs) {
                println!("Failure Case: Lookup Call");
                return false;
            }
        }
        true
    }
}

impl CallChecker<'_> {
    fn check_indirect_call(
        &self,
        state: &CallCheckLattice,
        target: &Value,
        _loc_idx: &LocIdx,
    ) -> bool {
        match target {
            Value::Reg(regnum, size) => {
                if let Some(CallCheckValue::FnPtr) = state.regs.get(regnum, size).v {
                    return true;
                }
            }
            Value::Mem(_, _) => return false,
            Value::Imm(_, _, _) => return true,
        }
        false
    }

    fn check_calltable_lookup(&self, state: &CallCheckLattice, memargs: &MemArgs) -> bool {
        // println!("Call Table Lookup: {:?}", memargs);
        match memargs {
            MemArgs::Mem3Args(
                MemArg::Reg(regnum1, ValSize::Size64),
                MemArg::Reg(regnum2, ValSize::Size64),
                MemArg::Imm(_, _, 8),
            ) => match (
                state.regs.get(regnum1, &ValSize::Size64).v,
                state.regs.get(regnum2, &ValSize::Size64).v,
            ) {
                (
                    Some(CallCheckValue::GuestTableBase),
                    Some(CallCheckValue::PtrOffset(DAV::Checked)),
                ) => return true,
                (
                    Some(CallCheckValue::PtrOffset(DAV::Checked)),
                    Some(CallCheckValue::GuestTableBase),
                ) => return true,
                (_x, Some(CallCheckValue::GuestTableBase))
                | (Some(CallCheckValue::GuestTableBase), _x) => return false,
                (_x, _y) => return true, // not a calltable lookup
            },
            _ => return true, //not a calltable lookup?
        }
    }
}
