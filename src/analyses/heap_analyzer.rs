use crate::lattices::reachingdefslattice::LocIdx;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::heaplattice::{HeapValueLattice, HeapLattice, HeapValue};
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Stmt, Value, MemArgs, MemArg};
use crate::utils::{LucetMetadata, get_rsp_offset};
use std::default::Default;
use crate::lattices::VarState;

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
        result.regs.rdi = HeapValueLattice::new(HeapValue::HeapBase);
        result
    }

    // fn aexec(&self, in_state : &mut HeapLattice, ir_instr : &Stmt, _loc_idx : &LocIdx) -> () {
    //     println!("Heap aexec: {:?}", ir_instr);
    //     match ir_instr{
    //         Stmt::Clear(dst) => in_state.set_to_bot(dst),
    //         Stmt::Unop(_, dst, src) => in_state.set(dst, self.aeval_unop(in_state, src)), 
    //         Stmt::Binop(opcode, dst, src1, src2) =>  self.aexec_binop(in_state, opcode, dst, src1, src2),//in_state.default_exec_binop(dst,src1,src2),
    //         Stmt::Call(_) => in_state.regs.clear_regs(),
    //         _ => ()
    //     }
    // }
    fn aexec_unop(&self, in_state : &mut HeapLattice, dst : &Value, src : &Value) -> (){
        in_state.set(dst, self.aeval_unop(in_state, src))
    }
}

pub fn is_globalbase_access(in_state : &HeapLattice, memargs : &MemArgs) -> bool {
    match memargs{
        MemArgs::Mem2Args(arg1, arg2) => 
            if let MemArg::Reg(regnum,size) = arg1{
                assert_eq!(size.to_u32(), 64);
                let base = in_state.regs.get(regnum);
                if let Some(v) = base.v {
                    if let HeapValue::HeapBase = v {return true}
                }
            },
        _ => return false
    };
    false
}

impl HeapAnalyzer{
    pub fn aeval_unop(&self, in_state : &HeapLattice, value : &Value) -> HeapValueLattice{
        match value{
            Value::Mem(memsize, memargs) => 
                if is_globalbase_access(in_state, memargs){
                    return HeapValueLattice::new(HeapValue::GlobalsBase)
                }

            Value::Reg(regnum, size) => 
                if size.to_u32() <= 32 {
                    return HeapValueLattice::new(HeapValue::Bounded4GB)
                } 
                else {
                    return in_state.regs.get(regnum)
                },
                
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
