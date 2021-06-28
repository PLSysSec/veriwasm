use crate::{analyses, checkers, ir, loaders};
use analyses::reaching_defs::analyze_reaching_defs;
use analyses::reaching_defs::ReachingDefnAnalyzer;
use analyses::{run_worklist, SwitchAnalyzer};
use checkers::resolve_jumps;
use ir::lift_cfg;
use ir::types::IRMap;
use ir::utils::has_indirect_jumps;
use loaders::utils::{deconstruct_elf, get_function_starts, get_symbol_addr, VW_Metadata};
use loaders::{ExecutableType, Loadable};
use yaxpeax_core::analyses::control_flow::{get_cfg, VW_CFG};
use yaxpeax_core::arch::x86_64::{x86_64Data, MergedContextTable};
use yaxpeax_core::memory::repr::process::ModuleData;

fn try_resolve_jumps(
    program: &ModuleData,
    contexts: &MergedContextTable,
    cfg: &VW_CFG,
    metadata: &VW_Metadata,
    irmap: &IRMap,
    _addr: u64,
) -> (VW_CFG, IRMap, i32, u32) {
    println!("Performing a reaching defs pass");
    let reaching_defs = analyze_reaching_defs(cfg, &irmap, metadata.clone());
    println!("Performing a jump resolution pass");
    let switch_analyzer = SwitchAnalyzer {
        metadata: metadata.clone(),
        reaching_defs: reaching_defs,
        reaching_analyzer: ReachingDefnAnalyzer {
            cfg: cfg.clone(),
            irmap: irmap.clone(),
        },
    };
    let switch_results = run_worklist(cfg, irmap, &switch_analyzer);
    let switch_targets = resolve_jumps(program, switch_results, &irmap, &switch_analyzer);

    let (new_cfg, still_unresolved) =
        get_cfg(program, contexts, cfg.entrypoint, Some(&switch_targets));
    let irmap = lift_cfg(&program, &new_cfg, &metadata);
    let num_targets = switch_targets.len();
    return (new_cfg, irmap, num_targets as i32, still_unresolved);
}

fn resolve_cfg(
    program: &ModuleData,
    contexts: &MergedContextTable,
    cfg: &VW_CFG,
    metadata: &VW_Metadata,
    orig_irmap: &IRMap,
    addr: u64,
) -> (VW_CFG, IRMap) {
    let (mut cfg, mut irmap, mut resolved_switches, mut still_unresolved) =
        try_resolve_jumps(program, contexts, cfg, metadata, orig_irmap, addr);
    while still_unresolved != 0 {
        let (new_cfg, new_irmap, new_resolved_switches, new_still_unresolved) =
            try_resolve_jumps(program, contexts, &cfg, metadata, &irmap, addr);
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
    program: &ModuleData,
    contexts: &MergedContextTable,
    metadata: &VW_Metadata,
    addr: u64,
) -> (VW_CFG, IRMap) {
    let (cfg, _) = get_cfg(program, contexts, addr, None);
    let irmap = lift_cfg(&program, &cfg, &metadata);
    if !has_indirect_jumps(&irmap) {
        return (cfg, irmap);
    }
    return resolve_cfg(program, contexts, &cfg, metadata, &irmap, addr);
}

pub fn get_one_resolved_cfg(
    binpath: &str,
    func: &str,
    program: &ModuleData,
    format: &ExecutableType,
) -> ((VW_CFG, IRMap), x86_64Data) {
    let metadata = format.load_metadata(program);

    let (_, sections, entrypoint, imports, exports, symbols) = deconstruct_elf(program);
    let text_section_idx = sections.iter().position(|x| x.name == ".text").unwrap();
    let x86_64_data = get_function_starts(entrypoint, symbols, imports, exports, text_section_idx);

    let addr = get_symbol_addr(symbols, func).unwrap();
    assert!(format.is_valid_func_name(&String::from(func)));
    println!("Generating CFG for: {:?}", func);
    return (
        fully_resolved_cfg(&program, &x86_64_data.contexts, &metadata, addr),
        x86_64_data,
    );
}
