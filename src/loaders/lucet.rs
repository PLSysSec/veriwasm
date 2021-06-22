use crate::loaders::utils::*;
use crate::utils::utils::deconstruct_elf;
use crate::utils::utils::get_symbol_addr;
use std::path::Path;
use yaxpeax_arch::Arch;
use yaxpeax_core::memory::reader;
use yaxpeax_core::memory::repr::process::{
    ELFExport, ELFImport, ELFSymbol, ModuleData, ModuleInfo,
};
use yaxpeax_core::memory::repr::FileRepr;
use yaxpeax_core::memory::MemoryRepr;
use yaxpeax_x86::long_mode::Arch as AMD64;
// use lucet_analyze::*;


pub fn load_lucet_program(binpath: &str) -> ModuleData {
    let program = yaxpeax_core::memory::reader::load_from_path(Path::new(binpath)).unwrap();
    if let FileRepr::Executable(program) = program {
        program
    } else {
        panic!("function:{} is not a valid path", binpath)
    }
}

pub fn load_lucet_metadata(program: &ModuleData) -> VW_Metadata {
    let (_, sections, entrypoint, imports, exports, symbols) = deconstruct_elf(program);

    let guest_table_0 = get_symbol_addr(symbols, "guest_table_0").unwrap();
    let lucet_tables = get_symbol_addr(symbols, "lucet_tables").unwrap();
    let lucet_probestack = get_symbol_addr(symbols, "lucet_probestack").unwrap();
    // println!(
    //     "guest_table_0 = {:x} lucet_tables = {:x} probestack = {:x}",
    //     guest_table_0, lucet_tables, lucet_probestack
    // );
    VW_Metadata {
        guest_table_0: guest_table_0,
        lucet_tables: lucet_tables,
        lucet_probestack: lucet_probestack,
    }
}

// func name is valid if:
// 1. is not probestack
pub fn is_valid_lucet_func_name(name: &String) -> bool {
    if name == "lucet_probestack" {
        return false;
    }
    true
}




pub fn get_lucet_func_signatures() -> FuncSignatures {
    unimplemented!();
}
