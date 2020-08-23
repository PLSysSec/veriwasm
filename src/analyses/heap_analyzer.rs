use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::heaplattice::{HeapValueLattice, HeapLattice, HeapValue};
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Stmt, Value};
use crate::utils::{LucetMetadata, get_rsp_offset};
use std::default::Default;

//Top level function
pub fn analyze_heap(cfg : &ControlFlowGraph<u64>, irmap : IRMap, metadata : LucetMetadata){
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

    // TODO - handle stack offset tracking
    fn aexec(&self, in_state : &mut HeapLattice, ir_instr : &Stmt) -> () {
        match ir_instr{
            Stmt::Clear(dst) => in_state.set_to_bot(dst),
            Stmt::Unop(_, dst, src) => in_state.set(dst, self.aeval_unop(in_state, src)), //in_state.set_to_bot(dst),
            Stmt::Binop(_, dst, _, _) =>  in_state.set_to_bot(dst),
            Stmt::Call(_) => in_state.regs.clear_regs(),
            _ => ()
        }
    }

    fn process_branch(&self, in_state : HeapLattice) -> Vec<HeapLattice>{
        vec![in_state.clone(), in_state.clone()]
    }
}

// Need to handle load from globals table
impl HeapAnalyzer{
    pub fn aeval_unop(&self, in_state : &HeapLattice, value : &Value) -> HeapValueLattice{
        match value{
                Value::Mem(memsize, memargs) => 
                match get_rsp_offset(memargs){ 
                    Some(offset) => in_state.stack.get(offset, memsize.to_u32()),
                    None => Default::default()
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
