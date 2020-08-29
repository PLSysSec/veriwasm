use crate::lattices::reachingdefslattice::LocIdx;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::heaplattice::{HeapValueLattice, HeapLattice, HeapValue};
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Stmt, Value, MemArgs, MemArg};
use crate::utils::{LucetMetadata, get_rsp_offset};
use std::default::Default;

//Top level function
pub fn analyze_heap(cfg : &ControlFlowGraph<u64>, irmap : &IRMap, metadata : LucetMetadata){
    run_worklist(cfg, irmap, HeapAnalyzer{metadata : metadata});    
}

pub struct HeapAnalyzer{
    metadata: LucetMetadata
}

impl AbstractAnalyzer<HeapLattice> for HeapAnalyzer {
    fn init_state(&self) -> HeapLattice {
        let mut result : HeapLattice = Default::default();
        result.regs.rdi = HeapValueLattice { v : Some(HeapValue::HeapBase)};
        result
    }

    fn aexec(&self, in_state : &mut HeapLattice, ir_instr : &Stmt, _loc_idx : &LocIdx) -> () {
        match ir_instr{
            Stmt::Clear(dst) => in_state.set_to_bot(dst),
            Stmt::Unop(_, dst, src) => in_state.set(dst, self.aeval_unop(in_state, src)), 
            Stmt::Binop(_, dst, src1, src2) =>  in_state.default_exec_binop(dst,src1,src2),
            Stmt::Call(_) => in_state.regs.clear_regs(),
            _ => ()
        }
    }
}

pub fn is_globalbase_access(in_state : &HeapLattice, memargs : &MemArgs) -> bool {
    match memargs{
        MemArgs::Mem2Args(arg1, arg2) => 
            if let MemArg::Reg(regnum,size) = arg1{
                assert_eq!(size.to_u32(), 64);
                let base = in_state.regs.get(regnum);
                if let Some(v) = base.v {
                    if let HeapValue::HeapBase = v {
                        return true
                    }
                    else{false}
                }
                else{false}
            } else {false},
        _ => false
    }
}

impl HeapAnalyzer{
    pub fn aeval_unop(&self, in_state : &HeapLattice, value : &Value) -> HeapValueLattice{
        match value{
                Value::Mem(memsize, memargs) => 
                match get_rsp_offset(memargs){ 
                    Some(offset) => in_state.stack.get(offset, memsize.to_u32()),
                    None => if is_globalbase_access(in_state, memargs){
                        HeapValueLattice{ v : Some(HeapValue::GlobalsBase)}
                    }
                    else {Default::default()}
                }

                Value::Reg(regnum, size) => {if size.to_u32() <= 32 {
                    HeapValueLattice{ v : Some(HeapValue::Bounded4GB)}} 
                    else {in_state.regs.get(regnum)} },
                    
                Value::Imm(_,_,immval) => 
                    if (*immval as u64) == self.metadata.guest_table_0 {
                        HeapValueLattice{ v : Some(HeapValue::GuestTable0)}
                    }
                    else {
                        if (*immval as u64) == self.metadata.lucet_tables {
                            HeapValueLattice{ v : Some(HeapValue::LucetTables)}
                        }
                        else{
                            if (*immval > 0) && (*immval < (1 << 32) ) {HeapValueLattice{ v : Some(HeapValue::Bounded4GB)}}
                            else {Default::default()}
                    }
                }
            }    
        }
    }
