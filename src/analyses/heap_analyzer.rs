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

    //TODO: complete aexec
    // - handle stack offset tracking
    fn aexec(&self, in_state : &mut HeapLattice, ir_instr : &Stmt) -> () {
        match ir_instr{
            Stmt::Clear(dst) => in_state.set_to_bot(dst),
            Stmt::Unop(_, dst, _) => in_state.set_to_bot(dst),
            Stmt::Binop(_, dst, _, _) =>  in_state.set_to_bot(dst),
            Stmt::Call(_) => in_state.regs.clear_regs(),
            _ => ()
        }
    }

    fn process_branch(&self, in_state : HeapLattice) -> Vec<HeapLattice>{
        // let output : Vec<StackGrowthLattice> = {in_state.clone(), in_state.clone()}
        let mut output = Vec::new();
        output.push(in_state.clone());
        output.push(in_state.clone());
        output
    }
}

// Need to handle mem[rsp + x]
// Need to handle load from globals table
// TODO: fix get_rsp_offset -- 
impl HeapAnalyzer{
    pub fn aeval(&self, in_state : &HeapLattice, value : &Value) -> HeapValueLattice{
        match value{
                Value::Mem(memargs) => 
                match get_rsp_offset(memargs){ 
                    Some(offset) => unimplemented!("Need memarg size"), //in_state.stack.get(offset),
                    None => Default::default()
                }
                // match memargs{
                //     MemArg::Mem1Arg(arg) => (),
                //     MemArg::Mem2Args(arg1, arg2) => (),
                //     _ => Default::default(),
                // },
                Value::Reg(regnum, size) => in_state.regs.get(regnum),
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
