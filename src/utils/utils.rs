use crate::{analyses, checkers, ir, loaders};
use analyses::jump_analyzer::analyze_jumps;
use analyses::jump_analyzer::SwitchAnalyzer;
use analyses::reaching_defs::analyze_reaching_defs;
use analyses::reaching_defs::ReachingDefnAnalyzer;
use checkers::jump_resolver::resolve_jumps;
use ir::types::IRMap;
use ir::utils::has_indirect_jumps;
use ir::x64::lift_cfg;
use loaders::utils::VW_Metadata;
use loaders::{ExecutableType, Loadable};
use yaxpeax_arch::Arch;
use yaxpeax_core::analyses::control_flow::{get_cfg, VW_CFG};
use yaxpeax_core::arch::x86_64::{x86_64Data, MergedContextTable};
use yaxpeax_core::arch::{BaseUpdate, Library, Symbol, SymbolQuery};
use yaxpeax_core::goblin::elf::program_header::ProgramHeader;
use yaxpeax_core::memory::repr::process::{
    ELFExport, ELFImport, ELFSection, ELFSymbol, ModuleData, ModuleInfo,
};
use yaxpeax_core::memory::MemoryRepr;
use yaxpeax_core::ContextWrite;
use yaxpeax_x86::long_mode::Arch as AMD64;

pub fn is_libcall(name: &String) -> bool {
    vec!["floor", "ceil", "trunc"].contains(&&name[..])
}

pub fn deconstruct_elf(
    program: &ModuleData,
) -> (
    &Vec<ProgramHeader>,
    &Vec<ELFSection>,
    &u64,
    &Vec<ELFImport>,
    &Vec<ELFExport>,
    &Vec<ELFSymbol>,
) {
    match (program as &dyn MemoryRepr<<AMD64 as Arch>::Address>).module_info() {
        Some(ModuleInfo::ELF(
            isa,
            _header,
            program_header,
            sections,
            entry,
            _relocs,
            imports,
            exports,
            symbols,
        )) => (program_header, sections, entry, imports, exports, symbols),
        Some(other) => {
            panic!("Module isn't an elf, but is a {:?}?", other);
        }
        None => {
            panic!("Module doesn't appear to be a binary yaxpeax understands.");
        }
    }
}

fn get_function_starts(
    entrypoint: &u64,
    symbols: &std::vec::Vec<ELFSymbol>,
    imports: &std::vec::Vec<ELFImport>,
    exports: &std::vec::Vec<ELFExport>,
    _text_section_idx: usize,
) -> x86_64Data {
    let mut x86_64_data = x86_64Data::default();

    // start queuing up places we expect to find functions
    x86_64_data.contexts.put(
        *entrypoint as u64,
        BaseUpdate::Specialized(yaxpeax_core::arch::x86_64::x86Update::FunctionHint),
    );

    // copy in symbols (not really necessary here)
    for sym in symbols {
        if sym.name != "" {
            x86_64_data.contexts.put(
                sym.addr as u64,
                BaseUpdate::DefineSymbol(Symbol(Library::This, sym.name.clone())),
            );
        }
    }

    //All symbols in text section should be function starts
    for sym in symbols {
        x86_64_data.contexts.put(
            sym.addr as u64,
            BaseUpdate::Specialized(yaxpeax_core::arch::x86_64::x86Update::FunctionHint),
        );
    }

    // and copy in names for imports
    for import in imports {
        x86_64_data.contexts.put(
            import.value as u64,
            BaseUpdate::DefineSymbol(Symbol(Library::Unknown, import.name.clone())),
        );
    }

    // exports are probably functions? hope for the best
    for export in exports {
        x86_64_data.contexts.put(
            export.addr as u64,
            BaseUpdate::Specialized(yaxpeax_core::arch::x86_64::x86Update::FunctionHint),
        );
    }
    x86_64_data
}

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
    let switch_results = analyze_jumps(cfg, &irmap, &switch_analyzer);
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

pub fn get_data(
    program: &ModuleData,
    format: &ExecutableType,
) -> (
    x86_64Data,
    Vec<(u64, std::string::String)>,
    (u64, u64),
    Vec<(u64, std::string::String)>,
) {
    let (_, sections, entrypoint, imports, exports, symbols) = deconstruct_elf(program);
    let text_section_idx = sections.iter().position(|x| x.name == ".text").unwrap();
    let mut x86_64_data =
        get_function_starts(entrypoint, symbols, imports, exports, text_section_idx);

    let plt_bounds = if let Some(plt_idx) = sections.iter().position(|x| x.name == ".plt") {
        let plt = sections.get(plt_idx).unwrap();
        (plt.start, plt.start + plt.size)
    } else {
        (0, 0)
    };

    let text_section = sections.get(text_section_idx).unwrap();

    let mut addrs: Vec<(u64, std::string::String)> = Vec::new();
    let mut all_addrs: Vec<(u64, std::string::String)> = Vec::new();

    while let Some(addr) = x86_64_data.contexts.function_hints.pop() {
        if let Some(symbol) = x86_64_data.symbol_for(addr) {
            all_addrs.push((addr, symbol.1.clone()));
        }
        if !((addr >= text_section.start) && (addr < (text_section.start + text_section.size))) {
            continue;
        }
        if let Some(symbol) = x86_64_data.symbol_for(addr) {
            if format.is_valid_func_name(&symbol.1) {
                addrs.push((addr, symbol.1.clone()));
            }
        }
    }
    (x86_64_data, addrs, plt_bounds, all_addrs)
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

//return addr of symbol if present, else None
pub fn get_symbol_addr(symbols: &Vec<ELFSymbol>, name: &str) -> Option<u64> {
    symbols
        .iter()
        .find(|sym| sym.name == name)
        .map(|sym| sym.addr)
}
