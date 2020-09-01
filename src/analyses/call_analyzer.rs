// use crate::analyses::VarStateAnalyzer;
use crate::lattices::calllattice::{CallCheckLattice, CallCheckValue, CallCheckValueLattice};
use crate::lattices::davlattice::DAV;
use crate::lifter::{MemArgs, MemArg, ValSize, Binopcode, IRMap, Stmt, Value};
use crate::utils::get_rsp_offset;
use crate::analyses::AnalysisResult;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lattices::reachingdefslattice::{ReachLattice, LocIdx};
use crate::utils::{LucetMetadata};
use std::default::Default;

//Top level function
pub fn analyze_calls(cfg : &ControlFlowGraph<u64>, irmap : &IRMap, metadata : LucetMetadata, reaching_defs : AnalysisResult<ReachLattice>){
    run_worklist(cfg, irmap, CallAnalyzer{metadata : metadata, reaching_defs : reaching_defs});    
}

pub struct CallAnalyzer{
    metadata: LucetMetadata,
    reaching_defs : AnalysisResult<ReachLattice>
}
// pub type CallAnalyzer = VarStateAnalyzer<CallCheckLattice>;

impl AbstractAnalyzer<CallCheckLattice> for CallAnalyzer {

    //TODO: complete this aexec function
    fn aexec(&self, in_state : &mut CallCheckLattice, ir_instr : &Stmt, loc_idx : &LocIdx) -> () {
        match ir_instr{
            Stmt::Clear(dst) => in_state.set_to_bot(dst),
            Stmt::Unop(_, dst, src) => in_state.set_to_bot(dst),//in_state.set(dst, self.aeval_unop(in_state, src)),
            Stmt::Binop(opcode, dst, src1, src2) =>  in_state.default_exec_binop(dst,src1,src2),
            Stmt::Call(_) => in_state.regs.clear_regs(),
            _ => ()
        }
    }

    //TODO: need to add final instruction to process_branch args
    //TODO: figure out how to extract zflag
    fn process_branch(&self, in_state : &CallCheckLattice, succ_addrs : &Vec<u64>) -> Vec<(u64,CallCheckLattice)>{
        if succ_addrs.len() == 2{
        succ_addrs.into_iter().map(|addr| (addr.clone(),in_state.clone()) ).collect()
        }
        else {succ_addrs.into_iter().map(|addr| (addr.clone(),in_state.clone()) ).collect()}
    }
}

pub fn is_table_size(in_state : &CallCheckLattice, memargs : &MemArgs) -> bool{
    match memargs{
        MemArgs::Mem2Args(MemArg::Reg(regnum1,_), MemArg::Imm(_,_,immval)) => 
            if let Some(CallCheckValue::LucetTablesBase) = in_state.regs.get(regnum1).v{
                return *immval == 8 
            },
        _ => return false
    }
    false
}

pub fn is_fn_ptr(in_state : &CallCheckLattice, memargs : &MemArgs) -> bool{
    match memargs{
        MemArgs::Mem3Args(MemArg::Reg(regnum1,_), MemArg::Reg(regnum2,_), MemArg::Imm(_,_,immval)) => 
            {
                match (in_state.regs.get(regnum1).v,in_state.regs.get(regnum2).v,immval){
                    (Some(CallCheckValue::GuestTableBase),Some(CallCheckValue::PtrOffset(DAV::Checked)),8) => return true,
                    (Some(CallCheckValue::PtrOffset(DAV::Checked)),Some(CallCheckValue::GuestTableBase),8) => return true,
                    _ => return false
                }
            },
        _ => return false
    }
}

impl CallAnalyzer{
    pub fn aeval_unop(&self, in_state : &CallCheckLattice, value : &Value) -> CallCheckValueLattice{
        match value{
                Value::Mem(memsize, memargs) => 
                if let Some(offset) = get_rsp_offset(memargs) { 
                    return in_state.stack.get(offset, memsize.to_u32())
                }
                else if is_table_size(in_state, memargs){
                    return CallCheckValueLattice{ v : Some(CallCheckValue::TableSize)} 
                }
                else if is_fn_ptr(in_state, memargs){
                    return CallCheckValueLattice{ v : Some(CallCheckValue::FnPtr)} 
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