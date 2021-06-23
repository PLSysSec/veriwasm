use crate::loaders::utils::*;
use crate::utils::utils::deconstruct_elf;
use crate::utils::utils::get_symbol_addr;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use lucet_module;
use std::io::{Cursor, Read};
use std::mem;
use std::path::Path;
use yaxpeax_arch::Address;
use yaxpeax_arch::Arch;
use yaxpeax_core::memory::reader;
use yaxpeax_core::memory::repr::process::{
    ELFExport, ELFImport, ELFSymbol, ModuleData, ModuleInfo, Segment,
};
use yaxpeax_core::memory::repr::FileRepr;
use yaxpeax_core::memory::MemoryRange;
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

pub fn load_lucet_module_data(program: &ModuleData) -> lucet_module::ModuleData {
    let (program_header, sections, entrypoint, imports, exports, symbols) =
        deconstruct_elf(program);
    let module_start: usize = get_symbol_addr(symbols, "lucet_module").unwrap() as usize;
    let module_size: usize = mem::size_of::<lucet_module::SerializedModule>();

    let buffer = read_module_buffer(program, module_start, module_size).unwrap();
    let mut rdr = Cursor::new(buffer);
    let module_data_ptr = rdr.read_u64::<LittleEndian>().unwrap();
    let module_data_len = rdr.read_u64::<LittleEndian>().unwrap();

    let module_data_buffer =
        read_module_buffer(program, module_data_ptr as usize, module_data_len as usize).unwrap();

    lucet_module::ModuleData::deserialize(module_data_buffer)
        .expect("ModuleData deserialization failure")
}

fn segment_for(program: &ModuleData, addr: usize) -> Option<&Segment> {
    for segment in program.segments.iter() {
        if addr >= segment.start && addr < (segment.start + segment.data.len()) {
            return Some(segment);
        }
    }
    None
}

// Finds and returns the data corresponding [addr..addr+size]
pub fn read_module_buffer(program: &ModuleData, addr: usize, size: usize) -> Option<&[u8]> {
    println!("Addr = {:x}", addr);
    for s in &program.segments{
        println!("{} {:x} {:x}", s.name, s.start, s.start + s.data.len());
    }
    let segment = segment_for(program, addr)?;
    let read_start = addr - segment.start;
    let read_end = read_start + size;
    Some(&segment.data[read_start..read_end])
}

pub fn get_lucet_func_signatures(program: &ModuleData) -> FuncSignatures {
    let lucet_module_data = load_lucet_module_data(program);
    println!("{:?}", lucet_module_data.signatures());
    println!("{:?}", lucet_module_data.function_info());
    // Vec::new()
    unimplemented!();
}
