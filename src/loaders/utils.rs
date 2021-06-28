#![allow(non_camel_case_types)]

use lucet_module::{Signature, ValueType};
use std::collections::HashMap;

use crate::ir::types::ValSize;
use crate::lattices::VarIndex;
use crate::lattices::X86Regs::*;
use crate::loaders::{ExecutableType, Loadable};

use yaxpeax_arch::Arch;
use yaxpeax_core::arch::x86_64::x86_64Data;
use yaxpeax_core::arch::{BaseUpdate, Library, Symbol, SymbolQuery};
use yaxpeax_core::goblin::elf::program_header::ProgramHeader;
use yaxpeax_core::memory::repr::process::{
    ELFExport, ELFImport, ELFSection, ELFSymbol, ModuleData, ModuleInfo,
};
use yaxpeax_core::memory::MemoryRepr;
use yaxpeax_core::ContextWrite;
use yaxpeax_x86::long_mode::Arch as AMD64;

#[derive(Clone, Debug)]
pub struct VW_Metadata {
    pub guest_table_0: u64,
    pub lucet_tables: u64,
    pub lucet_probestack: u64,
}

#[derive(Clone, Debug)]
pub struct VwFuncInfo {
    // Index -> Type
    pub signatures: Vec<Signature>,
    // Name -> Index
    pub indexes: HashMap<String, u32>,
}

// TODO: unify this with other register and stack variable slot representations
// RDI, RSI, RDX, RCX, R8, R9,
// 7,   6,   3,   2,   8,  9,    then stack slots

pub fn to_system_v(sig: &Signature) -> Vec<(VarIndex, ValSize)> {
    let mut arg_locs = Vec::new();
    let mut i_ctr = 0; // integer arg #
    let mut f_ctr = 0; // floating point arg #
    let mut stack_offset = 0;
    for arg in &sig.params {
        match arg {
            ValueType::I32 | ValueType::I64 => {
                let index = match i_ctr {
                    0 => VarIndex::Reg(Rdi),
                    1 => VarIndex::Reg(Rsi),
                    2 => VarIndex::Reg(Rdx),
                    3 => VarIndex::Reg(Rcx),
                    4 => VarIndex::Reg(R8),
                    5 => VarIndex::Reg(R9),
                    _ => {
                        if let ValueType::I32 = arg {
                            stack_offset += 4;
                        } else {
                            stack_offset += 8;
                        };
                        VarIndex::Stack(stack_offset)
                    }
                };
                i_ctr += 1;
                match arg {
                    ValueType::I32 => arg_locs.push((index, ValSize::Size32)),
                    ValueType::I64 => arg_locs.push((index, ValSize::Size64)),
                    _ => (),
                };
            }
            ValueType::F32 | ValueType::F64 => {
                f_ctr += 1;
            }
        }
    }
    return arg_locs;
}

//return addr of symbol if present, else None
pub fn get_symbol_addr(symbols: &Vec<ELFSymbol>, name: &str) -> Option<u64> {
    symbols
        .iter()
        .find(|sym| sym.name == name)
        .map(|sym| sym.addr)
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

pub fn get_function_starts(
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
        x86_64_data.contexts.put(
            sym.addr as u64,
            BaseUpdate::DefineSymbol(Symbol(Library::This, sym.name.clone())),
        );
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

pub fn get_data(
    program: &ModuleData,
    format: &ExecutableType,
) -> (x86_64Data, Vec<(u64, std::string::String)>, (u64, u64)) {
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
    while let Some(addr) = x86_64_data.contexts.function_hints.pop() {
        if !((addr >= text_section.start) && (addr < (text_section.start + text_section.size))) {
            continue;
        }
        if let Some(symbol) = x86_64_data.symbol_for(addr) {
            if format.is_valid_func_name(&symbol.1) {
                addrs.push((addr, symbol.1.clone()));
            }
        }
    }
    (x86_64_data, addrs, plt_bounds)
}
