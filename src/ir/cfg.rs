use crate::{analyses, checkers, ir, loaders};
use analyses::reaching_defs::analyze_reaching_defs;
use analyses::reaching_defs::ReachingDefnAnalyzer;
use analyses::{run_worklist, SwitchAnalyzer};
use checkers::resolve_jumps;
use ir::lift_cfg;
use ir::types::IRMap;
use ir::utils::has_indirect_jumps;
use loaders::types::{ExecutableType, VwMetadata, VwModule};
use loaders::utils::{deconstruct_elf, get_function_starts, get_symbol_addr};
use loaders::Loadable;
use yaxpeax_core::analyses::control_flow::{get_cfg, VW_CFG};
use yaxpeax_core::arch::x86_64::{x86_64Data, MergedContextTable};
use yaxpeax_core::memory::repr::process::ModuleData;

fn try_resolve_jumps(
    module: &VwModule,
    contexts: &MergedContextTable,
    cfg: &VW_CFG,
    irmap: &IRMap,
    _addr: u64,
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
    let irmap = lift_cfg(module, &new_cfg);
    let num_targets = switch_targets.len();
    return (new_cfg, irmap, num_targets as i32, still_unresolved);
}

fn resolve_cfg(
    module: &VwModule,
    contexts: &MergedContextTable,
    cfg: &VW_CFG,
    orig_irmap: &IRMap,
    addr: u64,
) -> (VW_CFG, IRMap) {
    let (mut cfg, mut irmap, mut resolved_switches, mut still_unresolved) =
        try_resolve_jumps(module, contexts, cfg, orig_irmap, addr);
    while still_unresolved != 0 {
        let (new_cfg, new_irmap, new_resolved_switches, new_still_unresolved) =
            try_resolve_jumps(module, contexts, &cfg, &irmap, addr);
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
) -> (VW_CFG, IRMap) {
    let (cfg, _) = get_cfg(&module.program, contexts, addr, None);
    let irmap = lift_cfg(module, &cfg);
    if !has_indirect_jumps(&irmap) {
        return (cfg, irmap);
    }
    return resolve_cfg(module, contexts, &cfg, &irmap, addr);
}

pub fn get_one_resolved_cfg(
    func: &str,
    module: &VwModule,
    format: &ExecutableType,
) -> ((VW_CFG, IRMap), x86_64Data) {
    let (_, sections, entrypoint, imports, exports, symbols) = deconstruct_elf(&module.program);
    let text_section_idx = sections.iter().position(|x| x.name == ".text").unwrap();
    let x86_64_data = get_function_starts(entrypoint, symbols, imports, exports, text_section_idx);

    let addr = get_symbol_addr(symbols, func).unwrap();
    assert!(format.is_valid_func_name(&String::from(func)));
    println!("Generating CFG for: {:?}", func);
    return (
        fully_resolved_cfg(module, &x86_64_data.contexts, addr),
        x86_64_data,
    );
}
