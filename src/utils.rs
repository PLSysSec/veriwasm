use crate::has_indirect_jumps;
use yaxpeax_core::arch::x86_64::MergedContextTable;
use crate::lifter::IRMap;
use crate::analyses::jump_analyzer::SwitchAnalyzer;
use crate::analyses::reaching_defs::ReachingDefnAnalyzer;
use crate::checkers::jump_resolver::resolve_jumps;
use crate::analyses::jump_analyzer::analyze_jumps;
use crate::analyses::reaching_defs::analyze_reaching_defs;
use crate::lifter::lift_cfg;
use yaxpeax_core::analyses::control_flow::VW_CFG;
use yaxpeax_core::analyses::control_flow::get_cfg;
use std::path::Path;
use yaxpeax_arch::Arch;
use yaxpeax_x86::long_mode::{Arch as AMD64};
use yaxpeax_core::arch::{Library, Symbol, BaseUpdate};
use yaxpeax_core::memory::repr::FileRepr;
use yaxpeax_core::memory::repr::process::{ModuleData, ModuleInfo, ELFExport, ELFImport, ELFSymbol};
use yaxpeax_core::memory::{MemoryRepr};
use yaxpeax_core::ContextWrite;
use yaxpeax_core::arch::x86_64::x86_64Data;
use yaxpeax_core::arch::SymbolQuery;
use crate::lifter::{MemArg, MemArgs};

pub fn load_program(binpath : &str) -> ModuleData{
    let program = yaxpeax_core::memory::reader::load_from_path(Path::new(binpath)).unwrap();
    let program = if let FileRepr::Executable(program) = program {
        program
    } else {
        panic!(format!("function:{} is not a valid path", binpath));
    };
    program
}

fn get_function_starts(entrypoint : &u64, 
    symbols : &std::vec::Vec<ELFSymbol>,
    imports : &std::vec::Vec<ELFImport>,
    exports : &std::vec::Vec<ELFExport>,
    text_section_idx : usize,
) -> x86_64Data{
    let mut x86_64_data = x86_64Data::default();

    // start queuing up places we expect to find functions
    x86_64_data.contexts.put(*entrypoint as u64, BaseUpdate::Specialized(
        yaxpeax_core::arch::x86_64::x86Update::FunctionHint
    ));

    // copy in symbols (not really necessary here)
    for sym in symbols {
        x86_64_data.contexts.put(sym.addr as u64, BaseUpdate::DefineSymbol(
            Symbol(Library::This, sym.name.clone())
        ));
    }

    //All symbols in text section should be function starts 
    for sym in symbols {
        // if sym.section_index == text_section_idx{
            println!("sym = {:?}", sym);
            x86_64_data.contexts.put(sym.addr as u64, BaseUpdate::Specialized(
                yaxpeax_core::arch::x86_64::x86Update::FunctionHint
            ));
        // }
    }

    // and copy in names for imports
    for import in imports {
        x86_64_data.contexts.put(import.value as u64, BaseUpdate::DefineSymbol(
            Symbol(Library::Unknown, import.name.clone())
        ));
    }

    // exports are probably functions? hope for the best
    for export in exports {
        x86_64_data.contexts.put(export.addr as u64, BaseUpdate::Specialized(
            yaxpeax_core::arch::x86_64::x86Update::FunctionHint
        ));
    }
    x86_64_data
}

// pub fn get_cfgs(binpath : &str) -> Vec<(String, VW_CFG)>{
//     let program = load_program(binpath);

//     // grab some details from the binary and panic if it's not what we expected
//     let (_, entrypoint, imports, exports, symbols) = match (&program as &dyn MemoryRepr<<AMD64 as Arch>::Address>).module_info() {
//         Some(ModuleInfo::ELF(isa, _, _, _sections, entry, _, imports, exports, symbols)) => {
//             (isa, entry, imports, exports, symbols)
//         }
//         Some(other) => {
//             panic!("{:?} isn't an elf, but is a {:?}?", binpath,other);
//         }
//         None => {
//             panic!("{:?} doesn't appear to be a binary yaxpeax understands.", binpath);
//         }
//     };

//     let mut x86_64_data = get_function_starts(entrypoint, symbols, imports, exports);

//     let mut cfgs : Vec<(String, VW_CFG)> = Vec::new(); 
//     while let Some(addr) = x86_64_data.contexts.function_hints.pop() {

//         if let Some(symbol) = x86_64_data.symbol_for(addr)
//         {
//         if is_valid_func_name(&symbol.1) { 
//             println!("Generating CFG for: {:?}", symbol.1);
//             let (new_cfg,_) = get_cfg(&program, &x86_64_data.contexts, addr, None);
//             cfgs.push((symbol.1.clone(), new_cfg));
//             }
//         }
//     }
//     cfgs
// }

fn try_resolve_jumps(program : &ModuleData,  contexts: &MergedContextTable, cfg : &VW_CFG, metadata : LucetMetadata, addr: u64) -> (VW_CFG,IRMap,i32,u32){
    let irmap = lift_cfg(&program, cfg, &metadata);
    let reaching_defs = analyze_reaching_defs(cfg, &irmap, metadata.clone());
    let switch_analyzer = SwitchAnalyzer{metadata : metadata.clone(), reaching_defs : reaching_defs, reaching_analyzer : ReachingDefnAnalyzer{}};
    let switch_results = analyze_jumps(cfg, &irmap, &switch_analyzer);
    let switch_targets = resolve_jumps(program, switch_results, &irmap, &switch_analyzer);
    
    let (new_cfg,still_unresolved) = get_cfg(program, contexts, cfg.entrypoint, Some(&switch_targets) );
    let irmap = lift_cfg(&program, &new_cfg, &metadata);
    let num_targets = switch_targets.len();
    return (new_cfg, irmap, num_targets as i32, still_unresolved);
}

fn resolve_cfg(program : &ModuleData, contexts: &MergedContextTable, cfg : &VW_CFG, metadata : LucetMetadata, addr: u64) -> (VW_CFG,IRMap){
    let mut last_switches = -1;
    let (mut cfg, mut irmap, mut resolved_switches, mut still_unresolved) = try_resolve_jumps(program, contexts, cfg, metadata.clone(), addr);
    while resolved_switches != last_switches && (still_unresolved != 0) {
        last_switches = resolved_switches;
        let (new_cfg, new_irmap, new_resolved_switches, new_still_unresolved) = try_resolve_jumps(program, contexts, &cfg, metadata.clone(), addr);
        cfg = new_cfg;
        irmap = new_irmap;
        if (new_resolved_switches == resolved_switches) && (new_still_unresolved != 0){
            panic!("Fixed Point Error");
        }
        resolved_switches = new_resolved_switches;
        still_unresolved = new_still_unresolved;
    } 
    println!("======> Resolved CFG");
    for block_addr in cfg.graph.nodes(){
        let dests: Vec<u64> = cfg.graph.neighbors(block_addr).collect();
        let out_addrs: Vec<std::string::String> = dests.clone().into_iter().map(|x| format!("{:x}", x)).rev().collect(); 
        println!("{:x} {:?}", block_addr, out_addrs);
    }
    assert_eq!(cfg.graph.node_count(), irmap.keys().len());
    assert_eq!(still_unresolved, 0);
    (cfg,irmap)
}

pub fn fully_resolved_cfg(program : &ModuleData, contexts: &MergedContextTable, metadata : LucetMetadata, addr: u64) -> (VW_CFG,IRMap){
    let (cfg,_) = get_cfg(program, contexts, addr, None);
    let irmap = lift_cfg(&program, &cfg, &metadata);
    if !has_indirect_jumps(&irmap){
        return (cfg,irmap);
    }
    return resolve_cfg(program, contexts, &cfg, metadata, addr)
}

pub fn get_resolved_cfgs(binpath : &str) -> Vec<(String, (VW_CFG,IRMap) )>{
    let program = load_program(binpath);
    let metadata = load_metadata(binpath);

    // grab some details from the binary and panic if it's not what we expected
    let (_, sections, entrypoint, imports, exports, symbols) = match (&program as &dyn MemoryRepr<<AMD64 as Arch>::Address>).module_info() {
        Some(ModuleInfo::ELF(isa, _, _, sections, entry, _, imports, exports, symbols)) => {
            (isa, sections, entry, imports, exports, symbols)
        }
        Some(other) => {
            panic!("{:?} isn't an elf, but is a {:?}?", binpath,other);
        }
        None => {
            panic!("{:?} doesn't appear to be a binary yaxpeax understands.", binpath);
        }
    };

    // println!("Symbols = {:?}", symbols);
    // for symbol in symbols{
    //     println!("symbol = {:?}", symbol.name);
    // }

   
    let text_section_idx = sections.iter().position(|x| x.name == ".text").unwrap();
    let text_section = sections.get(text_section_idx).unwrap();
    

    let mut x86_64_data = get_function_starts(entrypoint, symbols, imports, exports, text_section_idx);
    

    let mut cfgs : Vec<(String, (VW_CFG,IRMap) )> = Vec::new(); 
    while let Some(addr) = x86_64_data.contexts.function_hints.pop() {
        if !((addr >= text_section.start) && (addr <  (text_section.start + text_section.size))) { continue; }
        if let Some(symbol) = x86_64_data.symbol_for(addr){
        println!("Found maybe function: {:?} valid = {:?}", &symbol.1, is_valid_func_name(&symbol.1));
        if is_valid_func_name(&symbol.1) { 
            println!("Generating CFG for: {:?}", symbol.1);
            let (new_cfg,irmap) = fully_resolved_cfg(&program, &x86_64_data.contexts, metadata.clone(), addr);
            cfgs.push((symbol.1.clone(), (new_cfg,irmap) ));
            }
        }
    }
    cfgs
}


pub fn get_one_resolved_cfg(binpath : &str, func : &str) -> (VW_CFG,IRMap){
    let program = load_program(binpath);
    let metadata = load_metadata(binpath);

    // grab some details from the binary and panic if it's not what we expected
    let (_, sections, entrypoint, imports, exports, symbols) = match (&program as &dyn MemoryRepr<<AMD64 as Arch>::Address>).module_info() {
        Some(ModuleInfo::ELF(isa, _, _, sections, entry, _, imports, exports, symbols)) => {
            (isa, sections, entry, imports, exports, symbols)
        }
        Some(other) => {
            panic!("{:?} isn't an elf, but is a {:?}?", binpath,other);
        }
        None => {
            panic!("{:?} doesn't appear to be a binary yaxpeax understands.", binpath);
        }
    };

    let text_section_idx = sections.iter().position(|x| x.name == ".text").unwrap();
    let mut x86_64_data = get_function_starts(entrypoint, symbols, imports, exports, text_section_idx);
    // let symbol = x86_64_data.symbol_for(addr).unwrap();
    let addr = get_symbol_addr(symbols, func).unwrap();
    println!("Found maybe function: {:?} valid = {:?}", func, is_valid_func_name(&String::from(func)));
    assert!(is_valid_func_name(&String::from(func)));  
    println!("Generating CFG for: {:?}", func);
    return fully_resolved_cfg(&program, &x86_64_data.contexts, metadata.clone(), addr);
}




fn get_symbol_addr(symbols : &Vec<ELFSymbol>, name : &str)-> std::option::Option<u64> {
    let mut x = None;
    for symbol in symbols.iter(){
        if symbol.name == name{
            x = Some(symbol.addr);
        }
    }
    x
}

#[derive(Clone)]
pub struct LucetMetadata {
    pub guest_table_0: u64,
    pub lucet_tables: u64,
    pub lucet_probestack: u64
}


pub fn load_metadata(binpath : &str) -> LucetMetadata{
    let program = load_program(binpath);

    // grab some details from the binary and panic if it's not what we expected
    let (_, sections, entrypoint, imports, exports, symbols) = match (&program as &dyn MemoryRepr<<AMD64 as Arch>::Address>).module_info() {
        Some(ModuleInfo::ELF(isa, _, _, sections, entry, _, imports, exports, symbols)) => {
            (isa, sections, entry, imports, exports, symbols)
        }
        Some(other) => {
            panic!("{:?} isn't an elf, but is a {:?}?", binpath,other);
        }
        None => {
            panic!("{:?} doesn't appear to be a binary yaxpeax understands.", binpath);
        }
    };

    // let mut x86_64_data = get_function_starts(entrypoint, symbols, imports, exports);
    let guest_table_0 = get_symbol_addr(symbols, "guest_table_0").unwrap();
    let lucet_tables = get_symbol_addr(symbols, "lucet_tables").unwrap();
    let lucet_probestack = get_symbol_addr(symbols, "lucet_probestack").unwrap();
    println!("guest_table_0 = {:x} lucet_tables = {:x} probestack = {:x}", guest_table_0, lucet_tables, lucet_probestack);
    LucetMetadata {guest_table_0 : guest_table_0, lucet_tables : lucet_tables, lucet_probestack : lucet_probestack}
}


pub fn get_rsp_offset(memargs : &MemArgs) -> Option<i64>{
    match memargs{
        MemArgs::Mem1Arg(arg) => 
            if let MemArg::Reg(regnum,_) = arg{ 
                if *regnum == 4 {Some(0)} 
                else{None}
            }
            else {None},
        MemArgs::Mem2Args(arg1, arg2) => 
        if let MemArg::Reg(regnum, size) = arg1{
            if *regnum == 4{
                if let MemArg::Imm(imm_sign,_,offset) = arg2{Some(*offset)}
                else {None}
            }
            else {None}
        }
        else {None},
        _ => None
    }
}

// func name is valid if:
// 1. starts with guest_func_
// 2. ends in _# (where # is some number)
pub fn is_valid_func_name(name : &String) -> bool{
    if name.starts_with("guest_func_"){
        return true
    };
    let split_name : Vec<&str> = name.split("_").collect();
    if split_name.len() > 1 && split_name[split_name.len() - 1].parse::<u64>().is_ok(){
        return true
    };
    false
    
}



