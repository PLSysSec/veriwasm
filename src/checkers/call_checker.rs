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
        if let Stmt::Call(value) = ir_stmt{
            match value {
                Value::Reg(regnum, size) => 
                    if let Some(CallCheckValue::FnPtr) = state.regs.get(regnum).v{    
                        return true
                    },
                _ => ()
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

