use crate::analyses::jump_analyzer::SwitchAnalyzer;
use crate::analyses::{AbstractAnalyzer, AnalysisResult};
use crate::ir::types::{IRMap, Stmt, Value};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::lattices::switchlattice::{SwitchLattice, SwitchValue, SwitchValueLattice};
use std::collections::HashMap;
use yaxpeax_core::memory::repr::process::ModuleData;
use yaxpeax_core::memory::MemoryRepr;

fn load_target(program: &ModuleData, addr: u64) -> i64 {
    let b0 = program.read(addr).unwrap() as u32;
    let b1 = (program.read(addr + 1).unwrap() as u32) << 8;
    let b2 = (program.read(addr + 2).unwrap() as u32) << 16;
    let b3 = (program.read(addr + 3).unwrap() as u32) << 24;
    (b0 + b1 + b2 + b3) as i64
}

fn extract_jmp_targets(program: &ModuleData, aval: &SwitchValueLattice) -> Vec<i64> {
    let mut targets: Vec<i64> = Vec::new();
    match aval.v {
        Some(SwitchValue::JmpTarget(base, upper_bound)) => {
            for idx in 0..upper_bound {
                let addr = base + idx * 4;
                let target = load_target(program, addr.into());
                let resolved_target = ((base as i32) + (target as i32)) as i64;
                targets.push(resolved_target);
            }
        }
        _ => panic!("Jump Targets Broken, target = {:?}", aval.v),
    }
    targets
}

// addr -> vec of targets
pub fn resolve_jumps(
    program: &ModuleData,
    result: AnalysisResult<SwitchLattice>,
    irmap: &IRMap,
    analyzer: &SwitchAnalyzer,
) -> HashMap<u64, Vec<i64>> {
    let mut switch_targets: HashMap<u64, Vec<i64>> = HashMap::new();

    for (block_addr, mut state) in result.clone() {
        for (addr, ir_stmts) in irmap.get(&block_addr).unwrap() {
            for (idx, ir_stmt) in ir_stmts.iter().enumerate() {
                analyzer.aexec(
                    &mut state,
                    ir_stmt,
                    &LocIdx {
                        addr: *addr,
                        idx: idx as u32,
                    },
                );
            }
        }
    }

    for (block_addr, mut state) in result {
        for (addr, ir_stmts) in irmap.get(&block_addr).unwrap() {
            for (idx, ir_stmt) in ir_stmts.iter().enumerate() {
                match ir_stmt {
                    Stmt::Branch(_, Value::Reg(regnum, regsize)) => {
                        let aval = state.regs.get(regnum, regsize);
                        let targets = extract_jmp_targets(program, &aval);
                        switch_targets.insert(*addr, targets);
                    }
                    Stmt::Branch(_, Value::Mem(_, _)) => {
                        panic!("Illegal Jump!");
                    }
                    _ => (),
                }

                analyzer.aexec(
                    &mut state,
                    ir_stmt,
                    &LocIdx {
                        addr: *addr,
                        idx: idx as u32,
                    },
                );
            }
        }
    }
    switch_targets
}
