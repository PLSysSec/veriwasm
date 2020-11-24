use crate::ir_utils::{is_stack_access, is_mem_access};
use crate::lifter::{Stmt, Value, ValSize, MemArg, MemArgs, IRMap};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::heaplattice::{HeapLattice, HeapValue};
use crate::checkers::Checker;
use crate::analyses::{AnalysisResult, AbstractAnalyzer};
use crate::analyses::heap_analyzer::HeapAnalyzer;

pub struct HeapChecker<'a>{
    irmap : &'a  IRMap, 
    analyzer : &'a HeapAnalyzer
}

pub fn check_heap(result : AnalysisResult<HeapLattice>, 
    irmap : &IRMap, 
    analyzer : &HeapAnalyzer) -> bool{
    HeapChecker{irmap : irmap, analyzer : analyzer}.check(result)    
}

impl Checker<HeapLattice> for HeapChecker<'_> {
    fn check(&self, result : AnalysisResult<HeapLattice>) -> bool{
        self.check_state_at_statements(result)
    }

    fn irmap(&self) -> &IRMap {self.irmap}
    fn aexec(&self, state: &mut HeapLattice, ir_stmt: &Stmt, loc: &LocIdx){
        self.analyzer.aexec(state, ir_stmt, loc)
    }

    fn check_statement(&self, state : &HeapLattice, ir_stmt : &Stmt) -> bool {
        // println!("rdi = {:?} r15 = {:?} stack = {:?}", state.regs.rdi,
        // state.regs.r15, state.stack);
        // println!("Checking statement for heap {:?} r13 = {:?} stack[0x58] = {:?}", ir_stmt, state.regs.r13, state.stack.get(0x10,ValSize::Size64.to_u32() / 8));
        match ir_stmt{
            //1. Check that at each call rdi = HeapBase
            Stmt::Call(_) => {
            //  println!("=============== call rdi = {:?}", state.regs.rdi.v);
             match state.regs.rdi.v{
                 Some(HeapValue::HeapBase) => (),
                 _ => {println!("Call failure {:?}", state.stack.get(0,8)); return false}
             }},
             //2. Check that all load and store are safe
             Stmt::Unop(_, dst, src) => 
             if is_mem_access(dst){
                if !self.check_mem_access(state, dst){return false}
            }
            //stack read: probestack <= stackgrowth + c < 8K
            else if is_mem_access(src) {
                if !self.check_mem_access(state, src){return false}
            },
             _ => ()
        }
        true
    }
}

impl HeapChecker<'_> {
    fn check_global_access(&self, state : &HeapLattice, access: &Value) -> bool{
        if let Value::Mem(memsize, memargs) = access {
            match memargs{
                MemArgs::Mem1Arg(MemArg::Reg(regnum,ValSize::Size64)) => 
                    if let  Some(HeapValue::GlobalsBase) = state.regs.get(regnum,&ValSize::Size64).v { 
                        return true
                    }
                MemArgs::Mem2Args(MemArg::Reg(regnum,ValSize::Size64), MemArg::Imm(_,_,globals_offset)) => {
                    if let Some(HeapValue::GlobalsBase) = state.regs.get(regnum,&ValSize::Size64).v{    
                        return *globals_offset <= 4096
                    }
                },
                _ => return false
            }            
        }
        false
    }

    fn check_heap_access(&self, state : &HeapLattice, access: &Value) -> bool{
        // println!("ch");
        if let Value::Mem(size, memargs) = access {
            match memargs{
                // if only arg is heapbase
                MemArgs::Mem1Arg(MemArg::Reg(regnum,ValSize::Size64)) => 
                    if let Some(HeapValue::HeapBase) = state.regs.get(regnum,&ValSize::Size64).v {
                        return true
                },
                // if arg1 is heapbase and arg2 is bounded
                MemArgs::Mem2Args(MemArg::Reg(regnum,ValSize::Size64),memarg2) => 
                if let Some(HeapValue::HeapBase) = state.regs.get(regnum,&ValSize::Size64).v {
                    match memarg2{
                        MemArg::Reg(regnum2, size2) => 
                        if let Some(HeapValue::Bounded4GB) = state.regs.get(regnum2,size2).v {
                            return true
                        },
                        MemArg::Imm(_,_,v) => return *v <= 0xffffffff 
                    }
                }
                // if arg1 is heapbase and arg2 and arg3 are bounded || 
                // if arg1 is bounded and arg1 and arg3 are bounded
                MemArgs::Mem3Args(MemArg::Reg(regnum,ValSize::Size64),memarg2,memarg3) |  
                MemArgs::Mem3Args(memarg2,MemArg::Reg(regnum,ValSize::Size64),memarg3) => 
                if let Some(HeapValue::HeapBase) = state.regs.get(regnum,&ValSize::Size64).v {
                    match (memarg2,memarg3){
                        (MemArg::Reg(regnum2, size2),MemArg::Imm(_,_,v)) | (MemArg::Imm(_,_,v),MemArg::Reg(regnum2, size2))=> 
                        if let Some(HeapValue::Bounded4GB) = state.regs.get(regnum2, size2).v {
                            return *v <= 0xffffffff
                        },
                        (MemArg::Reg(regnum2, size2),MemArg::Reg(regnum3, size3)) => 
                            if let (Some(HeapValue::Bounded4GB),Some(HeapValue::Bounded4GB)) = (state.regs.get(regnum2,size2).v,state.regs.get(regnum3,size3).v){
                                return true
                            }
                        _ => () 
                    }
                },
                _ => return false
            }
        }
        false
    }

    fn check_metadata_access(&self, state : &HeapLattice, access: &Value) -> bool{
        //TODO: allow metadata access if global_table_base is either of the args
        if let Value::Mem(size, memargs) = access {
            match memargs{
                //Case 1: mem[globals_base]
                MemArgs::Mem1Arg(MemArg::Reg(regnum,ValSize::Size64)) => 
                    if let  Some(HeapValue::GlobalsBase) = state.regs.get(regnum,&ValSize::Size64).v { 
                        return true
                    }
                //Case 2: mem[lucet_tables + 8]
                MemArgs::Mem2Args(MemArg::Reg(regnum,ValSize::Size64), MemArg::Imm(_,_,8)) => {
                    if let Some(HeapValue::LucetTables) = state.regs.get(regnum,&ValSize::Size64).v{    
                        return true
                    }
                },
                MemArgs::Mem2Args(MemArg::Reg(regnum1,ValSize::Size64), MemArg::Reg(regnum2,ValSize::Size64)) => {
                    if let Some(HeapValue::GuestTable0) = state.regs.get(regnum1,&ValSize::Size64).v{    
                        return true
                    }
                    if let Some(HeapValue::GuestTable0) = state.regs.get(regnum2,&ValSize::Size64).v{    
                        return true
                    }
                },
                MemArgs::Mem3Args(MemArg::Reg(regnum1,ValSize::Size64),MemArg::Reg(regnum2,ValSize::Size64), MemArg::Imm(_,_,8)) 
                /*| MemArgs::MemScale(MemArg::Reg(regnum1,ValSize::Size64),MemArg::Reg(regnum2,ValSize::Size64), MemArg::Imm(_,_,4))*/ => {
                    match (state.regs.get(regnum1,&ValSize::Size64).v,state.regs.get(regnum2,&ValSize::Size64).v){
                        (Some(HeapValue::GuestTable0),_) => return true,
                        (_,Some(HeapValue::GuestTable0)) => return true,
                        _ => ()
                    }
                }
                _ => return false
            }       
        }
        false
    }

    //TODO: properly check jump table memory reads --- wire in jump analysis data
    fn check_jump_table_access(&self, state : &HeapLattice, access: &Value) -> bool{
        if let Value::Mem(size, memargs) = access {
            match memargs{
                MemArgs::MemScale(_,_,MemArg::Imm(_,_,4)) => return true,
                _ => return false
            }            
        }
        false
    }

    fn check_mem_access(&self, state : &HeapLattice, access: &Value) -> bool{
        //println!("Memory Access: {:?} {:?}", access, state.regs);
        // Case 1: its a stack access
        if is_stack_access(access) { return true}
        // Case 2: its a heap access
        if self.check_heap_access(state, access){ return true };
        // Case 3: its a metadata access
        if self.check_metadata_access(state, access){ return true };
        // Case 4: its a globals access
        if self.check_global_access(state, access){ return true };
        // Case 5: Jump table access
        if self.check_jump_table_access(state, access){ return true };
        // Case 6: its unknown
        println!("None of the memory accesses!");
        print_mem_access(state, access);
        return false
    }
   
}

pub fn memarg_repr(state: &HeapLattice, memarg: &MemArg) -> String{
    match memarg{
        MemArg::Reg(regnum,size) => format!("r{:?}: {:?}",regnum, state.regs.get(regnum, size).v),
        MemArg::Imm(_,_,x) => format!("{:?}", x),
    }
}

pub fn print_mem_access(state: &HeapLattice, access: &Value){
    if let Value::Mem(size, memargs) = access {
        match memargs{
            MemArgs::Mem1Arg(x) => println!("mem[{:?}]", memarg_repr(state, x)),
            MemArgs::Mem2Args(x,y) => println!("mem[{:?} + {:?}]", memarg_repr(state, x), memarg_repr(state, y)),
            MemArgs::Mem3Args(x,y,z) => println!("mem[{:?} + {:?} + {:?}]", memarg_repr(state, x), memarg_repr(state, y), memarg_repr(state, z)),
            MemArgs::MemScale(x,y,z) => println!("mem[{:?} + {:?} * {:?}]", memarg_repr(state, x), memarg_repr(state, y), memarg_repr(state, z)),
        }
    }
}

