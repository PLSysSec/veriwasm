use crate::lifter::IRBlock;
use crate::ir_utils::extract_stack_offset;
use crate::ir_utils::is_stack_access;
use crate::lattices::calllattice::{CallCheckLattice, CallCheckValue, CallCheckValueLattice};
use crate::lattices::davlattice::DAV;
use crate::lifter::{MemArgs, MemArg, Binopcode, IRMap, Value};
use crate::analyses::AnalysisResult;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lattices::reachingdefslattice::{ReachLattice};
use crate::utils::{LucetMetadata};
use std::default::Default;
use crate::lattices::VarState;

//Top level function
// pub fn analyze_calls(
//     cfg : &ControlFlowGraph<u64>, 
//     irmap : &IRMap, 
//     metadata : LucetMetadata, 
//     reaching_defs : AnalysisResult<ReachLattice>
//     ) -> AnalysisResult<CallCheckLattice>{
//     run_worklist(cfg, irmap, CallAnalyzer{metadata : metadata, reaching_defs : reaching_defs})    
// }

pub struct CallAnalyzer{
    pub metadata: LucetMetadata,
    pub reaching_defs : AnalysisResult<ReachLattice>
}

impl AbstractAnalyzer<CallCheckLattice> for CallAnalyzer {
    fn aexec_unop(&self, in_state : &mut CallCheckLattice, dst : &Value, src : &Value) -> (){
        in_state.set(dst, self.aeval_unop(in_state, src))
    }

    fn aexec_binop(&self, in_state : &mut CallCheckLattice, opcode : &Binopcode, dst: &Value, src1 : &Value, src2: &Value) -> (){
        in_state.set(dst, self.aeval_binop(in_state, opcode, src1, src2))
    }

    //TODO: need to add final instruction to process_branch args
    //TODO: figure out how to extract zflag
    fn process_branch(&self, irmap : &IRMap, in_state : &CallCheckLattice, succ_addrs : &Vec<u64>, addr : &u64) -> Vec<(u64,CallCheckLattice)>{
        if succ_addrs.len() == 2{
            let mut not_branch_state = in_state.clone();
            let mut branch_state = in_state.clone();
            //if zf = CheckFlag(regnum) and and state.get(regnum) == PtrOffset
            //=> state.set(regnum, PtrOffset(Checked)) 
            //TODO: set zf to checkflag
            // if let Some(SwitchValue::ZF(bound, regnum)) = not_branch_state.regs.zf.v{
            //     not_branch_state.regs.set(&regnum, SwitchValueLattice{v: Some(SwitchValue::UpperBound(bound))})
            // }
            vec![(succ_addrs[0].clone(),not_branch_state), (succ_addrs[1].clone(),branch_state)]
            //succ_addrs.into_iter().map(|addr| (addr.clone(),in_state.clone()) ).collect()
        }
        else {succ_addrs.into_iter().map(|addr| (addr.clone(),in_state.clone()) ).collect()}
    }
        // if succ_addrs.len() == 2{
        // succ_addrs.into_iter().map(|addr| (addr.clone(),in_state.clone()) ).collect()
        // }
        // else {succ_addrs.into_iter().map(|addr| (addr.clone(),in_state.clone()) ).collect()}
    // }
}

pub fn is_table_size(in_state : &CallCheckLattice, memargs : &MemArgs) -> bool{
    if let MemArgs::Mem2Args(MemArg::Reg(regnum1,_), MemArg::Imm(_,_,8)) = memargs{ 
        if let Some(CallCheckValue::LucetTablesBase) = in_state.regs.get(regnum1).v{
            return true 
        }
    }
    false
}

pub fn is_fn_ptr(in_state : &CallCheckLattice, memargs : &MemArgs) -> bool{
    if let MemArgs::Mem3Args(MemArg::Reg(regnum1,_), MemArg::Reg(regnum2,_), MemArg::Imm(_,_,immval))  = memargs{ 
        {
        match (in_state.regs.get(regnum1).v,in_state.regs.get(regnum2).v,immval){
            (Some(CallCheckValue::GuestTableBase),Some(CallCheckValue::PtrOffset(DAV::Checked)),8) => return true,
            (Some(CallCheckValue::PtrOffset(DAV::Checked)),Some(CallCheckValue::GuestTableBase),8) => return true,
            _ => return false
            }
        }
    }
    false
}

impl CallAnalyzer{
    pub fn aeval_unop(&self, in_state : &CallCheckLattice, value : &Value) -> CallCheckValueLattice{
        match value{
                Value::Mem(memsize, memargs) => 
                    if is_table_size(in_state, memargs){
                        return CallCheckValueLattice{ v : Some(CallCheckValue::TableSize)} 
                    }
                    else if is_fn_ptr(in_state, memargs){
                        return CallCheckValueLattice{ v : Some(CallCheckValue::FnPtr)} 
                    }
                    else if is_stack_access(value){
                        let offset = extract_stack_offset(memargs);
                        return in_state.stack.get(offset, memsize.to_u32())
                    },

                Value::Reg(regnum, size) => return in_state.regs.get(regnum),
                    
                Value::Imm(_,_,immval) => 
                    if (*immval as u64) == self.metadata.guest_table_0 {
                        return CallCheckValueLattice{ v : Some(CallCheckValue::GuestTableBase)}
                    }
                    else if (*immval as u64) == self.metadata.lucet_tables {
                        return CallCheckValueLattice{ v : Some(CallCheckValue::LucetTablesBase)}
                }
            } 
            Default::default()   
        }

        //checked_val << 4
        pub fn aeval_binop(&self, in_state : &CallCheckLattice, opcode : &Binopcode, src1 : &Value, src2: &Value) -> CallCheckValueLattice{
            if let Binopcode::Shl = opcode{
                if let (Value::Reg(regnum1, _), Value::Imm(_,_, 4)) = (src1,src2){
                    if let Some(CallCheckValue::CheckedVal) = in_state.regs.get(regnum1).v{
                        return CallCheckValueLattice{ v : Some(CallCheckValue::PtrOffset(DAV::Checked))}
                    } 
                    else {
                        //TODO: use proper reaching def here
                        return CallCheckValueLattice{ v : Some(CallCheckValue::PtrOffset(DAV::Unchecked(1)))}
                    }
                }
            }   
            Default::default()
        }
    }