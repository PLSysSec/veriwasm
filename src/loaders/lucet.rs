use crate::loaders::utils::*;
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

pub fn load_lucet_program(binpath: &str) -> ModuleData {
    let program = yaxpeax_core::memory::reader::load_from_path(Path::new(binpath)).unwrap();
    if let FileRepr::Executable(program) = program {
        program
    } else {
        panic!("function:{} is not a valid path", binpath)
    }
}

pub fn load_lucet_metadata(program: &ModuleData) -> VW_Metadata {
    // let program = load_program(binpath);

    // grab some details from the binary and panic if it's not what we expected
    let (_, _sections, _entrypoint, _imports, _exports, symbols) =
        match (program as &dyn MemoryRepr<<AMD64 as Arch>::Address>).module_info() {
            Some(ModuleInfo::ELF(isa, _, _, sections, entry, _, imports, exports, symbols)) => {
                (isa, sections, entry, imports, exports, symbols)
            }
            Some(other) => {
                panic!("Module isn't an elf, but is a {:?}?", other);
            }
            None => {
                panic!("Module doesn't appear to be a binary yaxpeax understands");
            }
        };

    let guest_table_0 = get_symbol_addr(symbols, "guest_table_0").unwrap();
    let lucet_tables = get_symbol_addr(symbols, "lucet_tables").unwrap();
    let lucet_probestack = get_symbol_addr(symbols, "lucet_probestack").unwrap();
    println!(
        "guest_table_0 = {:x} lucet_tables = {:x} probestack = {:x}",
        guest_table_0, lucet_tables, lucet_probestack
    );
    VW_Metadata {
        guest_table_0: guest_table_0,
        lucet_tables: lucet_tables,
        lucet_probestack: lucet_probestack,
    }
}
