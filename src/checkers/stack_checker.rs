use crate::ir_utils::{is_stack_access, get_imm_mem_offset};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lifter::{Stmt,Value, MemArgs, MemArg};
use crate::lifter::IRMap;
use crate::analyses::stack_analyzer::StackAnalyzer;
use crate::checkers::Checker;
use crate::analyses::{AnalysisResult};
use crate::lattices::stackgrowthlattice::StackGrowthLattice;
use crate::analyses::AbstractAnalyzer;

pub struct StackChecker<'a>{
    irmap : &'a  IRMap, 
    analyzer : &'a StackAnalyzer
}

pub fn check_stack(
    result : AnalysisResult<StackGrowthLattice>, 
    irmap : &IRMap, 
    analyzer : &StackAnalyzer) -> bool{
    StackChecker{irmap : irmap, analyzer : analyzer}.check(result)    
}

impl Checker<StackGrowthLattice> for StackChecker<'_> {
    fn check(&self, result : AnalysisResult<StackGrowthLattice>) -> bool{
        //println!("{:?}", result);
        // for (k,v) in result.iter(){
        //     println!("{:x} {:?}", k, v);
        // }
        self.check_state_at_statements(result)
    }

    fn irmap(&self) -> &IRMap {self.irmap}
    fn aexec(&self, state: &mut StackGrowthLattice, ir_stmt: &Stmt, loc: &LocIdx){
        //println!("stack aexec: {:x} {:?}", loc.addr, state.v);
        self.analyzer.aexec(state, ir_stmt, loc)
    }

    fn check_statement(&self, state : &StackGrowthLattice, ir_stmt : &Stmt) -> bool {
        //1, stackgrowth is never Bottom or >= 0
        //println!("Check Statement: {:?}", state.v);
        match state.v {
            None => {println!("Failure Case: Stackgrowth = None"); return false},
            Some((stackgrowth,_)) => { if stackgrowth > 0 {
                //println!("Failure Case: Stackgrowth is positive = {:?}",stackgrowth);
                return false
                }
            } 
        }

        // //2. Reads and writes are in bounds
        match ir_stmt {
            //encapsulates both load and store
            Stmt::Unop(_,dst,src) => 
            // stack write: probestack <= stackgrowth + c < 0
            if is_stack_access(dst){
                if !self.check_stack_write(state, dst){println!("check_stack_write failed"); return false}
            }
            //stack read: probestack <= stackgrowth + c < 8K
            else if is_stack_access(src) {
                if !self.check_stack_read(state, src){println!("check_stack_read failed"); return false}
            },
            _ => (),
        }
        
        // 3. For all rets stackgrowth = 0
        if let Stmt::Ret = ir_stmt{
            if let Some((stackgrowth,_)) = state.v {
                if stackgrowth != 0 {
                    println!("stackgrowth != 0 at ret: stackgrowth = {:?}", stackgrowth);
                    return false
                }
            }
        }
        
        true
    }
}

impl StackChecker<'_> {
    fn check_stack_read(&self, state : &StackGrowthLattice, src: &Value) -> bool{
        if let Value::Mem(size, memargs) = src {
            match memargs{
                MemArgs::Mem1Arg(memarg) => 
                    return (-state.get_probestack().unwrap() <= state.get_stackgrowth().unwrap()) && (state.get_stackgrowth().unwrap() <=8096),
                MemArgs::Mem2Args(memarg1, memarg2) => {
                    let offset = get_imm_mem_offset(memarg2);
                    return (-state.get_probestack().unwrap() <= state.get_stackgrowth().unwrap() + offset) && (state.get_stackgrowth().unwrap() <=8096)
                },
                _ => return false //stack accesses should never have 3 args
            }
        }
        panic!("Unreachable")
    }

    fn check_stack_write(&self, state : &StackGrowthLattice, dst: &Value) -> bool{
        if let Value::Mem(size, memargs) = dst {
            match memargs{
                MemArgs::Mem1Arg(memarg) => 
                {
                    // println!("{:?} {:?}", state.get_probestack().unwrap(), state.get_stackgrowth());
                    return (-state.get_probestack().unwrap() <= state.get_stackgrowth().unwrap()) && (state.get_stackgrowth().unwrap() <=0)},
                MemArgs::Mem2Args(memarg1, memarg2) => {
                    let offset = get_imm_mem_offset(memarg2);
                    // println!("{:?} {:?} {:?}", state.get_probestack().unwrap(), state.get_stackgrowth(), offset);
                    return (-state.get_probestack().unwrap() <= state.get_stackgrowth().unwrap() + offset) && (state.get_stackgrowth().unwrap() <=0)
                },
                _ => return false //stack accesses should never have 3 args
            }
        }
        panic!("Unreachable")
    }
}
