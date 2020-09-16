use crate::lifter::{Stmt, Value, ValSize, MemArg, MemArgs};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::analyses::call_analyzer::CallAnalyzer;
use crate::lifter::IRMap;
use crate::checkers::Checker;
use crate::analyses::{AnalysisResult};
use crate::lattices::calllattice::{CallCheckLattice, CallCheckValue};
use crate::analyses::AbstractAnalyzer;
use crate::lattices;
use crate::lattices::davlattice::{DAV};

pub struct CallChecker<'a>{
    irmap : &'a  IRMap, 
    analyzer : &'a CallAnalyzer
}

// TODO
// if not has indirect_calls:
//      return true     
pub fn check_calls(result : AnalysisResult<CallCheckLattice>,
    irmap : &IRMap, 
    analyzer : &CallAnalyzer) -> bool{
    CallChecker{irmap, analyzer}.check(result)    
}

impl Checker<CallCheckLattice> for CallChecker<'_> {
    fn check(&self, result : AnalysisResult<CallCheckLattice>) -> bool{
        self.check_state_at_statements(result)
    }

    fn check_state(&self, state : &CallCheckLattice) -> bool {
        true
    }
}

impl CallChecker<'_> {
    fn check_state_at_statements(&self, result : AnalysisResult<CallCheckLattice>) -> bool{
        for (block_addr,mut state) in result {
            for (addr,ir_stmts) in self.irmap.get(&block_addr).unwrap(){
                for (idx,ir_stmt) in ir_stmts.iter().enumerate(){
                    self.analyzer.aexec(&mut state, ir_stmt, &LocIdx {addr : *addr, idx : idx as u32});
                    if !self.check_state(&state){
                        return false
                    }
                }
            }
        }
        true
    }
    // TODO check lookups
    fn check_statement(&self, state : &CallCheckLattice, ir_stmt : &Stmt) -> bool {
        if let Stmt::Call(value) = ir_stmt{
            match value {
                Value::Reg(regnum, size) => 
                    if let Some(CallCheckValue::FnPtr) = state.regs.get(regnum).v{    
                        return true
                    },
                _ => ()//return true
            }
        }

        // Check that lookup is using resolved DAV
        if let Stmt::Unop(_,_,Value::Mem(_,memargs)) = ir_stmt{
            match memargs{
                MemArgs::Mem3Args(MemArg::Reg(regnum1,ValSize::Size64),MemArg::Reg(regnum2,ValSize::Size64), MemArg::Imm(_,_,8)) =>
                match (state.regs.get(regnum1).v,state.regs.get(regnum2).v){
                    (Some(CallCheckValue::GuestTableBase),Some(CallCheckValue::PtrOffset(DAV::Checked))) => return true,
                    (Some(CallCheckValue::PtrOffset(DAV::Checked)),Some(CallCheckValue::GuestTableBase)) => return true,
                    _ => ()
                }
                _ => ()
            }
        }

        
        false
    }
}


