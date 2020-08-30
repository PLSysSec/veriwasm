use crate::lifter::{MemArgs, MemArg, ValSize, Binopcode, IRMap, Stmt, Value};
use crate::utils::get_rsp_offset;
use crate::analyses::AnalysisResult;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::switchlattice::{SwitchLattice, SwitchValueLattice, SwitchValue};
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lattices::reachingdefslattice::{ReachLattice, LocIdx};
use crate::utils::{LucetMetadata};
use std::default::Default;

//Top level function
pub fn analyze_jumps(cfg : &ControlFlowGraph<u64>, irmap : &IRMap, metadata : LucetMetadata, reaching_defs : AnalysisResult<ReachLattice>){
    run_worklist(cfg, irmap, SwitchAnalyzer{metadata : metadata, reaching_defs : reaching_defs});    
}

pub struct SwitchAnalyzer{
    metadata: LucetMetadata,
    reaching_defs : AnalysisResult<ReachLattice>
}

impl AbstractAnalyzer<SwitchLattice> for SwitchAnalyzer {
    fn init_state(&self) -> SwitchLattice {
        Default::default()
    }

    fn aexec(&self, in_state : &mut SwitchLattice, ir_instr : &Stmt, loc_idx : &LocIdx) -> () {
        match ir_instr{
            Stmt::Clear(dst) => in_state.set_to_bot(dst),
            Stmt::Unop(_, dst, src) => in_state.set(dst, self.aeval_unop(in_state, src)),
            Stmt::Binop(opcode, dst, src1, src2) =>  in_state.set(dst, self.aeval_binop(in_state, opcode, src1, src2)),
            Stmt::Call(_) => in_state.regs.clear_regs(),
            _ => ()
        }
    }
}

//TODO: implement process_bounds for switch analyzer

impl SwitchAnalyzer{
    fn aeval_unop_mem(&self, in_state : &SwitchLattice, memargs : &MemArgs, memsize : &ValSize)-> SwitchValueLattice {
        if let Some(offset) = get_rsp_offset(memargs){
            in_state.stack.get(offset, memsize.to_u32())
        }
            else{
                if let MemArgs::MemScale(MemArg::Reg(regnum1,_), MemArg::Reg(regnum2,_), MemArg::Imm(_,_,immval) ) = memargs{
                    if let (Some(SwitchValue::SwitchBase(base)),Some(SwitchValue::UpperBound(bound)),4) =   (in_state.regs.get(regnum1).v,in_state.regs.get(regnum2).v,immval){
                        SwitchValueLattice{v: Some(SwitchValue::JmpOffset(base,bound))}
                    }
                    else{Default::default()}
                }
                else{Default::default()}
        }
    }

    // 1. if unop is a constant, set as constant -- done
    // 2. if reg, return reg -- done
    // 3. if stack access, return stack access -- done
    // 4. x = mem[switch_base + offset * 4]
    pub fn aeval_unop(&self, in_state : &SwitchLattice, src : &Value) -> SwitchValueLattice{
        match src{
            Value::Mem(memsize, memargs) => self.aeval_unop_mem(in_state, memargs, memsize),
            Value::Reg(regnum, size) => in_state.regs.get(regnum),
            Value::Imm(_,_,immval) => SwitchValueLattice{v: Some(SwitchValue::SwitchBase(*immval as u32))}
            }   
        }

    // 1. x = switch_base + offset
    pub fn aeval_binop(&self, in_state : &SwitchLattice, opcode : &Binopcode, src1 : &Value, src2: &Value) -> SwitchValueLattice{
        if let Binopcode::Add = opcode{
            if let (Value::Reg(regnum1, _), Value::Reg(regnum2, _)) = (src1,src2){
                match (in_state.regs.get(regnum1).v,in_state.regs.get(regnum2).v){
                    (Some(SwitchValue::SwitchBase(base)),Some(SwitchValue::JmpOffset(_,offset))) =>
                        {SwitchValueLattice{v: Some(SwitchValue::JmpTarget(base,offset))}},
                    (Some(SwitchValue::JmpOffset(_, offset)),Some(SwitchValue::SwitchBase(base))) =>
                        {SwitchValueLattice{v: Some(SwitchValue::JmpTarget(base,offset))}},
                    _ => Default::default()
                }
            }
            else {Default::default()}
            }   
        else {Default::default()}
        }

    }
