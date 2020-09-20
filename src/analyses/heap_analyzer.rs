use crate::analyses::AnalysisResult;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::heaplattice::{HeapValueLattice, HeapLattice, HeapValue};
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Value, MemArgs, MemArg, ValSize};
use crate::utils::{LucetMetadata};
use crate::ir_utils::{is_stack_access, extract_stack_offset};
use std::default::Default;
use crate::lattices::VarState;

//Top level function
// pub fn analyze_heap(cfg : &ControlFlowGraph<u64>, irmap : &IRMap, metadata : LucetMetadata) -> AnalysisResult<HeapLattice>{
//     run_worklist(cfg, irmap, HeapAnalyzer{metadata : metadata})    
// }

pub struct HeapAnalyzer{
    pub metadata: LucetMetadata
}

impl AbstractAnalyzer<HeapLattice> for HeapAnalyzer {
    fn init_state(&self) -> HeapLattice {
        let mut result : HeapLattice = Default::default();
        result.regs.rdi = HeapValueLattice::new(HeapValue::HeapBase);
        result
    }

    fn aexec_unop(&self, in_state : &mut HeapLattice, dst : &Value, src : &Value) -> (){
        let v = self.aeval_unop(in_state, src);
        println!("dst = {:?} = {:?}", dst, v);
        in_state.set(dst, v)
    }
}

pub fn is_globalbase_access(in_state : &HeapLattice, memargs : &MemArgs) -> bool {
    if let MemArgs::Mem2Args(arg1, arg2) = memargs{ 
        if let MemArg::Reg(regnum,size) = arg1{
            assert_eq!(size.to_u32(), 64);
            let base = in_state.regs.get(regnum);
            if let Some(v) = base.v {
                if let HeapValue::HeapBase = v {return true}
            }
        }
    };
    false
}

impl HeapAnalyzer{
    pub fn aeval_unop(&self, in_state : &HeapLattice, value : &Value) -> HeapValueLattice{
        match value{
            Value::Mem(memsize, memargs) => {
                if is_globalbase_access(in_state, memargs){
                    return HeapValueLattice::new(HeapValue::GlobalsBase)
                }
                // put stack access here?
                if is_stack_access(value){
                    let offset = extract_stack_offset(memargs);
                    let v = in_state.stack.get(offset, memsize.to_u32());
                    println!("stack load({:?}) = {:?}", offset, v);
                    return v
                }
            }

            Value::Reg(regnum, size) => {
                if let ValSize::SizeOther = size {return Default::default()};
                if size.to_u32() <= 32 {
                    return HeapValueLattice::new(HeapValue::Bounded4GB)
                } 
                else {
                    return in_state.regs.get(regnum)
                }},
                
            Value::Imm(_,_,immval) => 
                if (*immval as u64) == self.metadata.guest_table_0 {
                    return HeapValueLattice::new(HeapValue::GuestTable0) 
                }
                else if (*immval as u64) == self.metadata.lucet_tables {
                    return HeapValueLattice::new(HeapValue::LucetTables)
                }
                else if (*immval > 0) && (*immval < (1 << 32) ) {
                    return HeapValueLattice::new(HeapValue::Bounded4GB)
                }
            }
                Default::default()
        }
    }
