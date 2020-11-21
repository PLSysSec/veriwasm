use crate::checkers::Checker;
use crate::lifter::{Stmt, Value, ValSize, MemArg, MemArgs, IRMap};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::calllattice::{CallCheckLattice, CallCheckValue};
use crate::lattices::davlattice::{DAV};
use crate::analyses::call_analyzer::CallAnalyzer;
use crate::analyses::{AnalysisResult};
use crate::analyses::AbstractAnalyzer;

pub struct CallChecker<'a>{
    irmap : &'a  IRMap, 
    analyzer : &'a CallAnalyzer
}
 
pub fn check_calls(result : AnalysisResult<CallCheckLattice>,
    irmap : &IRMap, 
    analyzer : &CallAnalyzer) -> bool{
    CallChecker{irmap, analyzer}.check(result)    
}

impl Checker<CallCheckLattice> for CallChecker<'_> {
    fn check(&self, result : AnalysisResult<CallCheckLattice>) -> bool{
        self.check_state_at_statements(result)
    }

    fn irmap(&self) -> &IRMap {self.irmap}
    fn aexec(&self, state: &mut CallCheckLattice, ir_stmt: &Stmt, loc: &LocIdx){
        self.analyzer.aexec(state, ir_stmt, loc)
    }

    // TODO check lookups
    fn check_statement(&self, state : &CallCheckLattice, ir_stmt : &Stmt) -> bool {
        //1. Check that all indirect calls use resolved function pointer
        if let Stmt::Call(v) = ir_stmt{
            if !check_indirect_call(state, v){
                println!("Failure Case: Indirect Call"); 
                return false
            }
        }
        

        // 2. Check that lookup is using resolved DAV
        if let Stmt::Unop(_,_,Value::Mem(_,memargs)) = ir_stmt{
            if !check_calltable_lookup(state, memargs){
                println!("Failure Case: Lookup Call"); 
                return false
            }
        }

        
        true
    }
}

fn check_indirect_call(state: &CallCheckLattice, target: &Value) -> bool {
    match target{
        Value::Reg(regnum, size) => 
            if let Some(CallCheckValue::FnPtr) = state.regs.get(regnum, size).v{    
                return true
        },
        Value::Mem(_,_) => return false,
        Value::Imm(_,_,_) => return true //TODO: check that this is in our set of target funcs
    }
    false
}

fn check_calltable_lookup(state: &CallCheckLattice, memargs: &MemArgs) -> bool {
    // println!("Call Table Lookup: {:?}", memargs);
    match memargs{
        MemArgs::Mem3Args(MemArg::Reg(regnum1,ValSize::Size64),MemArg::Reg(regnum2,ValSize::Size64), MemArg::Imm(_,_,8)) =>
        match (state.regs.get(regnum1,&ValSize::Size64).v,state.regs.get(regnum2,&ValSize::Size64).v){
            (Some(CallCheckValue::GuestTableBase),Some(CallCheckValue::PtrOffset(DAV::Checked))) => return true,
            (Some(CallCheckValue::PtrOffset(DAV::Checked)),Some(CallCheckValue::GuestTableBase)) => return true,
            (_x,Some(CallCheckValue::GuestTableBase)) | (Some(CallCheckValue::GuestTableBase),_x) => { return false},
            (_x,_y) => return true // not a calltable lookup
        }
        _ => return true //not a calltable lookup?
    }
}
