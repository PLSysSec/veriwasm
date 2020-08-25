use std::path::Path;
use yaxpeax_arch::Arch;
use yaxpeax_arch::AddressDisplay;
use yaxpeax_x86::long_mode::{Arch as AMD64};
use yaxpeax_core::arch::{Function, Library, Symbol, BaseUpdate};
use yaxpeax_core::memory::repr::FileRepr;
use yaxpeax_core::memory::repr::process::{ModuleData, ModuleInfo, ELFExport, ELFImport, ELFSymbol};
use yaxpeax_core::memory::{MemoryRepr};
use yaxpeax_core::ContextWrite;
use yaxpeax_core::arch::x86_64::x86_64Data;
use yaxpeax_core::analyses::control_flow;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph; 
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

    // and copy in names for imports
    for import in imports {
        x86_64_data.contexts.put(import.value as u64, BaseUpdate::DefineSymbol(
            Symbol(Library::Unknown, import.name.clone())
        ));
    }

    // exports are probably functions? hopve for the best
    for export in exports {
        x86_64_data.contexts.put(export.addr as u64, BaseUpdate::Specialized(
            yaxpeax_core::arch::x86_64::x86Update::FunctionHint
        ));
    }
    x86_64_data
}

fn function_cfg_for_addr(program: &ModuleData, 
    data: &mut x86_64Data, 
    addr: <AMD64 as Arch>::Address) -> ControlFlowGraph<<AMD64 as Arch>::Address> {
    if !data.contexts.functions.borrow().contains_key(&addr) {
        data.contexts.put(
            addr, BaseUpdate::DefineFunction(
                Function::of(
                    format!("function:{}", addr.show()),
                    vec![],
                    vec![],
                )
            )
        );

        control_flow::explore_all(
            program,
            &mut data.contexts,
            &mut data.cfg,
            vec![addr],
            &yaxpeax_core::arch::x86_64::analyses::all_instruction_analyses
        );
    }
    data.cfg.get_function(addr, &*data.contexts.functions.borrow())
}


pub fn get_cfgs(binpath : &str) -> Vec<(String, ControlFlowGraph<u64>)>{
    let program = load_program(binpath);

    // grab some details from the binary and panic if it's not what we expected
    let (_, entrypoint, imports, exports, symbols) = match (&program as &dyn MemoryRepr<<AMD64 as Arch>::Address>).module_info() {
        Some(ModuleInfo::ELF(isa, _, _, _sections, entry, _, imports, exports, symbols)) => {
            (isa, entry, imports, exports, symbols)
        }
        Some(other) => {
            panic!("{:?} isn't an elf, but is a {:?}?", binpath,other);
        }
        None => {
            panic!("{:?} doesn't appear to be a binary yaxpeax understands.", binpath);
        }
    };

    let mut x86_64_data = get_function_starts(entrypoint, symbols, imports, exports);

    let mut cfgs : Vec<(String, ControlFlowGraph<u64>)> = Vec::new(); 
    while let Some(addr) = x86_64_data.contexts.function_hints.pop() {
        // let function_cfg = function_cfg_for_addr(&program, &mut x86_64_data,
        // addr);
        // let func_name =
        // x86_64_data.contexts.function_at(addr).unwrap().name();
        if let Some(symbol) = x86_64_data.symbol_for(addr)
        {
        // println!("Generating CFG for: {:?}", symbol.1);
        cfgs.push((symbol.1.clone(), function_cfg_for_addr(&program, &mut x86_64_data, addr)));
        }
    }
    cfgs

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
    pub lucet_tables: u64
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
    println!("guest_table_0 = {:x} lucet_tables = {:x}", guest_table_0, lucet_tables);
    LucetMetadata {guest_table_0 : guest_table_0, lucet_tables : lucet_tables}
    // for symbol in symbols.iter(){
    //     if symbol.name == "guest_table_0"{
    //         println!("{:?} @ {:x} in section {:?}", symbol.name, symbol.addr, symbol.section_index);
    //     }
    //     if symbol.name == "lucet_tables"{
    //         println!("{:?} @ {:x} in section {:?}", symbol.name, symbol.addr, symbol.section_index);
    //     }

    //     // println!("{:?}",  sections[symbol.section_index].start);
        
    // }
    // println!("{:?}", symbols)
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
