// use lucet_module::ModuleData;
use crate::lattices::switchlattice::SwitchValueLattice;
use crate::lattices::switchlattice::SwitchValue;
use crate::analyses::jump_analyzer::SwitchAnalyzer;
use crate::lattices::switchlattice::SwitchLattice;
use crate::checkers::Checker;
use crate::lifter::{Stmt, Value, ValSize, MemArg, MemArgs, IRMap};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::davlattice::{DAV};
use crate::analyses::{AnalysisResult};
use crate::analyses::AbstractAnalyzer;
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
    println!("========================Extracting Jump Targets!=====================");
    println!("aval = {:?}", aval);
    let mut targets: Vec<i64> = Vec::new();
    match aval.v{
        Some(SwitchValue::JmpTarget(base, upper_bound)) => {
            for idx in 0..upper_bound {
                let addr = base + idx * 4; 
                let target = load_target(program, addr.into());
                targets.push(target);
            }
        },
        _ => panic!("Jump Targets Broken")
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
    for (block_addr, mut state) in result {
        for (addr,ir_stmts) in irmap.get(&block_addr).unwrap(){
            for (idx,ir_stmt) in ir_stmts.iter().enumerate(){
                if(*addr >= 0x384fd && *addr <= 0x38518){
                println!("------------\n{:x} {:?} rcx = {:?} rdx = {:?} r15 = {:?} zf = {:?}", addr, ir_stmt,state.regs.rcx, state.regs.rdx, state.regs.r15, state.regs.zf );
                }
                match ir_stmt {
                    Stmt::Branch(_, Value::Reg(regnum,regsize)) => {
                        let aval = state.regs.get(regnum);
                        println!("extracting jmp target @ {:x}", addr);
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



// impl Checker<SwitchLattice> for CallChecker<'_> {
//     fn check(&self, result : AnalysisResult<SwitchLattice>) -> bool{
//         //self.check_state_at_statements(result)
//         unimplemented!("Check not implemented for jump resolver")
//     }

//     fn irmap(&self) -> &IRMap {self.irmap}
//     fn aexec(&self, state: &mut SwitchLattice, ir_stmt: &Stmt, loc: &LocIdx){
//         self.analyzer.aexec(state, ir_stmt, loc)
//     }

//     // TODO check lookups
//     fn check_statement(&self, state : &SwitchLattice, ir_stmt : &Stmt) -> bool {
//         unimplemented!("Check statement is not implemented for jump")
//     }
// }
