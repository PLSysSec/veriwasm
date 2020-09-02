use crate::lifter::{MemArgs, MemArg, ValSize, Binopcode, IRMap, Stmt, Value};
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::switchlattice::{SwitchLattice, SwitchValueLattice, SwitchValue};
use crate::analyses::{AbstractAnalyzer, run_worklist, AnalysisResult};
use crate::lattices::reachingdefslattice::{ReachLattice, LocIdx};
use crate::utils::{LucetMetadata, get_rsp_offset};
use std::default::Default;
use crate::lattices::VarState;

//Top level function
pub fn analyze_jumps(cfg : &ControlFlowGraph<u64>, irmap : &IRMap, metadata : LucetMetadata, reaching_defs : AnalysisResult<ReachLattice>){
    run_worklist(cfg, irmap, SwitchAnalyzer{metadata : metadata, reaching_defs : reaching_defs});    
}

pub struct SwitchAnalyzer{
    metadata: LucetMetadata,
    reaching_defs : AnalysisResult<ReachLattice>
}

impl AbstractAnalyzer<SwitchLattice> for SwitchAnalyzer {
    fn aexec_unop(&self, in_state : &mut SwitchLattice, dst : &Value, src : &Value) -> (){
        in_state.set(dst, self.aeval_unop(in_state, src))
    }

    fn aexec_binop(&self, in_state : &mut SwitchLattice, opcode : &Binopcode, dst: &Value, src1 : &Value, src2: &Value) -> (){
        in_state.set(dst, self.aeval_binop(in_state, opcode, src1, src2))
    }
}

impl SwitchAnalyzer{
    fn aeval_unop_mem(&self, in_state : &SwitchLattice, memargs : &MemArgs, memsize : &ValSize)-> SwitchValueLattice {
        if let Some(offset) = get_rsp_offset(memargs){
            return in_state.stack.get(offset, memsize.to_u32())
        }
        if let MemArgs::MemScale(MemArg::Reg(regnum1,_), MemArg::Reg(regnum2,_), MemArg::Imm(_,_,immval) ) = memargs{
            if let (Some(SwitchValue::SwitchBase(base)),Some(SwitchValue::UpperBound(bound)),4) =   (in_state.regs.get(regnum1).v,in_state.regs.get(regnum2).v,immval){
                return SwitchValueLattice::new(SwitchValue::JmpOffset(base,bound))
            }
        }
        Default::default()
    }

    // 1. if unop is a constant, set as constant -- done
    // 2. if reg, return reg -- done
    // 3. if stack access, return stack access -- done
    // 4. x = mem[switch_base + offset * 4]
    pub fn aeval_unop(&self, in_state : &SwitchLattice, src : &Value) -> SwitchValueLattice{
        match src{
            Value::Mem(memsize, memargs) => self.aeval_unop_mem(in_state, memargs, memsize),
            Value::Reg(regnum, size) => in_state.regs.get(regnum),
            Value::Imm(_,_,immval) => SwitchValueLattice::new(SwitchValue::SwitchBase(*immval as u32))
            }   
        }

    // 1. x = switch_base + offset
    pub fn aeval_binop(&self, in_state : &SwitchLattice, opcode : &Binopcode, src1 : &Value, src2: &Value) -> SwitchValueLattice{
        if let Binopcode::Add = opcode{
            if let (Value::Reg(regnum1, _), Value::Reg(regnum2, _)) = (src1,src2){
                match (in_state.regs.get(regnum1).v,in_state.regs.get(regnum2).v){
                    (Some(SwitchValue::SwitchBase(base)),Some(SwitchValue::JmpOffset(_,offset))) =>
                        return SwitchValueLattice::new(SwitchValue::JmpTarget(base,offset)),
                    (Some(SwitchValue::JmpOffset(_, offset)),Some(SwitchValue::SwitchBase(base))) =>
                        return SwitchValueLattice::new(SwitchValue::JmpTarget(base,offset)),
                    _ => return Default::default()
                };
            }
        }   
        Default::default()
    }

    //TODO: need to add final instruction to process_branch args
    //TODO: figure out how to extract zflag
    fn process_branch(&self, in_state : &SwitchLattice, succ_addrs : &Vec<u64>) -> Vec<(u64,SwitchLattice)>{
        if succ_addrs.len() == 2{
        succ_addrs.into_iter().map(|addr| (addr.clone(),in_state.clone()) ).collect()
        }
        else {succ_addrs.into_iter().map(|addr| (addr.clone(),in_state.clone()) ).collect()}
    }
}
