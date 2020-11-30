use crate::lattices::stacklattice::StackSlot;
use crate::analyses::reaching_defs::ReachingDefnAnalyzer;
use yaxpeax_core::analyses::control_flow::VW_CFG;
use crate::lifter::{MemArgs, MemArg, ValSize, Binopcode, IRMap, Value};
use crate::lattices::switchlattice::{SwitchLattice, SwitchValueLattice, SwitchValue};
use crate::analyses::{AbstractAnalyzer, run_worklist, AnalysisResult, analyze_block};
use crate::lattices::reachingdefslattice::{ReachLattice, LocIdx};
use crate::utils::{LucetMetadata, get_rsp_offset};
use std::default::Default;
use crate::lattices::VarState;

//Top level function
pub fn analyze_jumps(
    cfg : &VW_CFG, 
    irmap : &IRMap, 
    switch_analyzer : &SwitchAnalyzer
    ) -> AnalysisResult<SwitchLattice>{

    run_worklist(cfg, irmap, switch_analyzer)    
}

pub struct SwitchAnalyzer{
    pub metadata: LucetMetadata,
    pub reaching_defs : AnalysisResult<ReachLattice>,
    pub reaching_analyzer : ReachingDefnAnalyzer,
}

impl AbstractAnalyzer<SwitchLattice> for SwitchAnalyzer {
    fn aexec_unop(&self, in_state : &mut SwitchLattice, dst : &Value, src : &Value, loc_idx : &LocIdx) -> (){
        // println!("exec unop: dst = {:?} rcx = {:?}", dst, in_state.regs.rcx);
        in_state.set(dst, self.aeval_unop(in_state, src))
    }

    fn aexec_binop(&self, in_state : &mut SwitchLattice, opcode : &Binopcode, dst: &Value, src1 : &Value, src2: &Value, loc_idx : &LocIdx) -> (){
        if let Binopcode::Cmp = opcode{
            // println!("CMP: {:?} = {:?} {:?}", dst, src1, src2);
            // match (in_state.,in_state.regs.get(regnum2).v){
            match (src1,src2){
                (Value::Reg(regnum,_),Value::Imm(_,_,imm)) | (Value::Imm(_,_,imm),Value::Reg(regnum,_)) =>
                    in_state.regs.zf = SwitchValueLattice::new(SwitchValue::ZF(*imm as u32, *regnum)),
                _ => ()
            }
        }
            //self.aeval_binop(in_state, opcode, src1, src2);
            // in_state.set(Value::Reg(), self.aeval_binop(in_state, opcode,
            // src1, src2))
        match opcode{
            Binopcode::Cmp => (),
            Binopcode::Test => (),
            _ => in_state.set(dst, self.aeval_binop(in_state, opcode, src1, src2))
        }
    }

    fn process_branch(&self, irmap : &IRMap, in_state : &SwitchLattice, succ_addrs : &Vec<u64>, addr : &u64) -> Vec<(u64,SwitchLattice)>{
        let defs_state = self.reaching_defs.get(addr).unwrap();
        // if *addr == 0x1055bb || *addr == 0x1055cc || *addr == 0x1055de || *addr == 0x105814 || *addr == 0x001056b9{
        //     println!("Start of {:x}: Analysis: mem[0x98] = {:?}, mem[0x44] = {:?}", addr, defs_state.stack.map.get(&(0x10 + defs_state.stack.offset)), defs_state.stack.map.get(&(0x64 + defs_state.stack.offset)));
        // }// println!("{:x}: Analysis: stack = {:?}", addr, defs_state.stack);
        if succ_addrs.len() == 2{
            let mut not_branch_state = in_state.clone();
            let mut branch_state = in_state.clone();
            if let Some(SwitchValue::ZF(bound, regnum)) = not_branch_state.regs.zf.v{
                not_branch_state.regs.set(&regnum, &ValSize::Size64, SwitchValueLattice{v: Some(SwitchValue::UpperBound(bound))});
                let defs_state = self.reaching_defs.get(addr).unwrap();
                let ir_block = irmap.get(addr).unwrap();
                let defs_state = analyze_block(&self.reaching_analyzer, defs_state, ir_block);
                // println!("End of {:x}: Analysis: mem[0x98] = {:?}, mem[0x44] = {:?}", addr, defs_state.stack.map.get(&(0x10 + defs_state.stack.offset)), defs_state.stack.map.get(&(0x64 + defs_state.stack.offset)));

                
                let checked_defs = defs_state.regs.get(&regnum, &ValSize::Size64);
                //propagate bound across registers with the same reaching def
                for idx in 0..15{
                    if idx != regnum{
                        let reg_def = defs_state.regs.get(&idx, &ValSize::Size64);
                        if (!reg_def.is_empty()) && (reg_def == checked_defs){
                            not_branch_state.regs.set(&idx, &ValSize::Size64, SwitchValueLattice{v: Some(SwitchValue::UpperBound(bound))}); 
                        }
                    }
                }
                //propagate bound across stack slots with the same upper bound
                for (stack_offset, stack_slot) in defs_state.stack.map.iter(){
                    if !checked_defs.is_empty() && (stack_slot.value == checked_defs){
                        let v = SwitchValueLattice{v: Some(SwitchValue::UpperBound(bound))};
                        let vv = StackSlot{size: stack_slot.size, value : v};
                        not_branch_state.stack.map.insert(*stack_offset, vv);
                    }
                }
            }
            branch_state.regs.zf = Default::default();
            not_branch_state.regs.zf = Default::default();
            vec![(succ_addrs[0].clone(),not_branch_state), (succ_addrs[1].clone(),branch_state)]
        }
        else {succ_addrs.into_iter().map(|addr| (addr.clone(),in_state.clone()) ).collect()}
    }
}

impl SwitchAnalyzer{
    fn aeval_unop_mem(&self, in_state : &SwitchLattice, memargs : &MemArgs, memsize : &ValSize)-> SwitchValueLattice {
        if let Some(offset) = get_rsp_offset(memargs){
            return in_state.stack.get(offset, memsize.to_u32() / 8)
        }
        if let MemArgs::MemScale(MemArg::Reg(regnum1,size1), MemArg::Reg(regnum2,size2), MemArg::Imm(_,_,immval) ) = memargs{
            if let (Some(SwitchValue::SwitchBase(base)),Some(SwitchValue::UpperBound(bound)),4) = (in_state.regs.get(regnum1,size1).v,in_state.regs.get(regnum2,size2).v,immval){
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
            Value::Reg(regnum, size) => in_state.regs.get(regnum, size),
            Value::Imm(_,_,immval) => SwitchValueLattice::new(SwitchValue::SwitchBase(*immval as u32))
            }   
        }

    // 1. x = switch_base + offset
    pub fn aeval_binop(&self, in_state : &SwitchLattice, opcode : &Binopcode, src1 : &Value, src2: &Value) -> SwitchValueLattice{
        
        if let Binopcode::Add = opcode{
            if let (Value::Reg(regnum1, size1), Value::Reg(regnum2, size2)) = (src1,src2){
                match (in_state.regs.get(regnum1,size1).v,in_state.regs.get(regnum2,size2).v){
                    (Some(SwitchValue::SwitchBase(base)),Some(SwitchValue::JmpOffset(_,offset))) 
                    | (Some(SwitchValue::JmpOffset(_, offset)),Some(SwitchValue::SwitchBase(base))) =>
                    {
                        return SwitchValueLattice::new(SwitchValue::JmpTarget(base,offset))
                    },
                    _ => return Default::default()
                };
            }
        }   
        Default::default()
    }
}
