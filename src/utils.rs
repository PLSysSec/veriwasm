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
use yaxpeax_core::arch::FunctionQuery;
use yaxpeax_core::ContextRead;
use yaxpeax_core::arch::AddressNamer;
use yaxpeax_core::arch::SymbolQuery;

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

