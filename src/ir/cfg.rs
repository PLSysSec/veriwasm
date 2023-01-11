use crate::{analyses, checkers, ir, loaders};
use analyses::reaching_defs::analyze_reaching_defs;
use analyses::reaching_defs::ReachingDefnAnalyzer;
use analyses::{run_worklist, SwitchAnalyzer};
use checkers::resolve_jumps;
use ir::lift_cfg;
use ir::types::*;
use loaders::types::VwModule;
use yaxpeax_core::analyses::control_flow::{get_cfg, VW_CFG};
use yaxpeax_core::arch::x86_64::MergedContextTable;

pub fn has_indirect_jumps(irmap: &IRMap) -> bool {
    for (_block_addr, ir_block) in irmap {
        for (_addr, ir_stmts) in ir_block {
            for (_idx, ir_stmt) in ir_stmts.iter().enumerate() {
                match ir_stmt {
                    Stmt::Branch(_, Value::Reg(_, _)) | Stmt::Branch(_, Value::Mem(_, _)) => {
                        return true
                    }
                    _ => (),
                }
            }
        }
    }
    false
}

fn try_resolve_jumps(
    module: &VwModule,
    contexts: &MergedContextTable,
    cfg: &VW_CFG,
    irmap: &IRMap,
    _addr: u64,
    strict: bool,
) -> (VW_CFG, IRMap, i32, u32) {
    println!("Performing a reaching defs pass");
    let reaching_defs = analyze_reaching_defs(cfg, &irmap, module.metadata.clone());
    println!("Performing a jump resolution pass");
    let switch_analyzer = SwitchAnalyzer {
        metadata: module.metadata.clone(),
        reaching_defs: reaching_defs,
        reaching_analyzer: ReachingDefnAnalyzer {
            cfg: cfg.clone(),
            irmap: irmap.clone(),
        },
    };
    let switch_results = run_worklist(cfg, irmap, &switch_analyzer);
    let switch_targets = resolve_jumps(&module.program, switch_results, &irmap, &switch_analyzer);

    let (new_cfg, still_unresolved) = get_cfg(
        &module.program,
        contexts,
        cfg.entrypoint,
        Some(&switch_targets),
    );
    let irmap = lift_cfg(module, &new_cfg, strict);
    let num_targets = switch_targets.len();
    return (new_cfg, irmap, num_targets as i32, still_unresolved);
}

fn resolve_cfg(
    module: &VwModule,
    contexts: &MergedContextTable,
    cfg: &VW_CFG,
    orig_irmap: &IRMap,
    addr: u64,
    strict: bool,
) -> (VW_CFG, IRMap) {
    let (mut cfg, mut irmap, mut resolved_switches, mut still_unresolved) =
        try_resolve_jumps(module, contexts, cfg, orig_irmap, addr, strict);
    while still_unresolved != 0 {
        let (new_cfg, new_irmap, new_resolved_switches, new_still_unresolved) =
            try_resolve_jumps(module, contexts, &cfg, &irmap, addr, strict);
        cfg = new_cfg;
        irmap = new_irmap;
        if (new_resolved_switches == resolved_switches) && (new_still_unresolved != 0) {
            panic!("Fixed Point Error");
        }
        resolved_switches = new_resolved_switches;
        still_unresolved = new_still_unresolved;
    }
    assert_eq!(cfg.graph.node_count(), irmap.keys().len());
    assert_eq!(still_unresolved, 0);
    (cfg, irmap)
}

pub fn fully_resolved_cfg(
    module: &VwModule,
    contexts: &MergedContextTable,
    addr: u64,
    strict: bool,
) -> (VW_CFG, IRMap) {
    let (cfg, _) = get_cfg(&module.program, contexts, addr, None);
    let irmap = lift_cfg(module, &cfg, strict);
    if !has_indirect_jumps(&irmap) {
        return (cfg, irmap);
    }
    return resolve_cfg(module, contexts, &cfg, &irmap, addr, strict);
}
