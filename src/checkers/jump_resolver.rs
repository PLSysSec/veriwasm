use crate::lattices::switchlattice::{SwitchValueLattice, SwitchValue, SwitchLattice};
use crate::analyses::jump_analyzer::SwitchAnalyzer;
use crate::lifter::{Stmt, Value, ValSize, MemArg, MemArgs, IRMap};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::analyses::{AbstractAnalyzer, AnalysisResult};
use std::collections::HashMap;
use yaxpeax_core::memory::{MemoryRepr, MemoryRange};
use yaxpeax_core::memory::repr::process::ModuleData;


pub struct JumpResolver<'a>{
    irmap : &'a  IRMap, 
    analyzer : &'a SwitchAnalyzer
}
 
fn load_target(program : &ModuleData, addr: u64) -> i64{
    let b0 = (program.read(addr).unwrap() as u32);
    let b1 = (program.read(addr + 1).unwrap() as u32) << 8;
    let b2 = (program.read(addr + 2).unwrap() as u32) << 16;
    let b3 = (program.read(addr + 3).unwrap() as u32) << 24;
    (b0 + b1 + b2 + b3) as i64
}

fn extract_jmp_targets(program : &ModuleData, aval : &SwitchValueLattice) -> Vec<i64>{
    let mut targets: Vec<i64> = Vec::new();
    match aval.v{
        Some(SwitchValue::JmpTarget(base, upper_bound)) => {
            for idx in 0..upper_bound {
                let addr = base + idx * 4; 
                let target = load_target(program, addr.into());
                let resolved_target = ((base as i32) + (target as i32)) as i64;
                // println!("Resolved Target to {:x} + {:x} = {:x}", base, target, ((base as i32) + (target as i32)) as u64);
                targets.push(resolved_target);
            }
        },
        _ => panic!("Jump Targets Broken, target = {:?}", aval.v)
    }
    targets
}

// addr -> vec of targets
pub fn resolve_jumps(
    program : &ModuleData,
    result : AnalysisResult<SwitchLattice>,
    irmap : &IRMap, 
    analyzer : &SwitchAnalyzer) -> HashMap<u64, Vec<i64>>    {
    let mut switch_targets: HashMap<u64, Vec<i64>> = HashMap::new();

    for (block_addr, mut state) in result.clone() {
        for (addr,ir_stmts) in irmap.get(&block_addr).unwrap(){
            for (idx,ir_stmt) in ir_stmts.iter().enumerate(){
                // println!("{:x}: rcx = {:?}", addr, state.regs.rcx);
                analyzer.aexec(&mut state, ir_stmt, &LocIdx {addr : *addr, idx : idx as u32});
            }
        }
    }

    for (block_addr, mut state) in result {
        // println!("{:x}: rcx = {:?}", block_addr, state.regs.rcx);
        for (addr,ir_stmts) in irmap.get(&block_addr).unwrap(){
            for (idx,ir_stmt) in ir_stmts.iter().enumerate(){
                // if(*addr >= 0x0001bf31 && *addr <= 0x0001bf39){
                //     println!("------------\n{:x} {:?} rax = {:?} rbx = {:?} r15 = {:?}", addr, ir_stmt,state.regs.rax, state.regs.rbx, state.regs.r15);
                // }
                match ir_stmt {
                    Stmt::Branch(_, Value::Reg(regnum,regsize)) => {
                        let aval = state.regs.get(regnum, regsize);
                        println!("extracting jmp target @ {:x}", addr);
                        // println!("0x{:x}  rax = {:?} rbx = {:?} r15 = {:?}", addr, state.regs.rax, state.regs.rbx, state.regs.r15);
                        let targets = extract_jmp_targets(program, &aval);
                        switch_targets.insert(*addr, targets);
                    },
                    Stmt::Branch(_, Value::Mem(_,_)) => {
                        panic!("Illegal Jump!");
                    },
                    _ => ()
                }
                
                analyzer.aexec(&mut state, ir_stmt, &LocIdx {addr : *addr, idx : idx as u32});
                
            }
        }
    }   
    switch_targets 
}

