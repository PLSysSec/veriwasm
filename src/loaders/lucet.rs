use crate::loaders;
use byteorder::{LittleEndian, ReadBytesExt};
// use loaders::utils::*;
use loaders::types::{VwFuncInfo, VwMetadata, VwModule};
use loaders::utils::{deconstruct_elf, get_symbol_addr};
use lucet_module;
use std::collections::HashMap;
use std::io::Cursor;
use std::mem;
use std::path::Path;
use yaxpeax_core::memory::repr::process::{ModuleData, Segment};
use yaxpeax_core::memory::repr::FileRepr;

fn load_lucet_metadata(program: &ModuleData) -> VwMetadata {
    let (_, sections, entrypoint, imports, exports, symbols) = deconstruct_elf(program);

    let guest_table_0 = get_symbol_addr(symbols, "guest_table_0").unwrap();
    let lucet_tables = get_symbol_addr(symbols, "lucet_tables").unwrap();
    let lucet_probestack = get_symbol_addr(symbols, "lucet_probestack").unwrap();
    VwMetadata {
        guest_table_0: guest_table_0,
        lucet_tables: lucet_tables,
        lucet_probestack: lucet_probestack,
    }
}

pub fn load_lucet_program(binpath: &str) -> VwModule {
    let program = yaxpeax_core::memory::reader::load_from_path(Path::new(binpath)).unwrap();
    if let FileRepr::Executable(program) = program {
        let metadata = load_lucet_metadata(&program);
        VwModule { program, metadata }
    } else {
        panic!("function:{} is not a valid path", binpath)
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
    let segment = segment_for(program, addr)?;
    let read_start = addr - segment.start;
    let read_end = read_start + size;
    Some(&segment.data[read_start..read_end])
}

pub fn get_lucet_func_signatures(program: &ModuleData) -> VwFuncInfo {
    let lucet_module_data = load_lucet_module_data(program);
    for signature in lucet_module_data.signatures() {
        println!("{:?}", signature);
    }
    let mut indexes = HashMap::new();
    for func_info in lucet_module_data.function_info() {
        println!("{:?}", func_info);
        indexes.insert(
            func_info.name.unwrap().to_string(),
            func_info.signature.as_u32(),
        );
    }
    // Vec::new()
    VwFuncInfo {
        signatures: lucet_module_data.signatures().to_vec(),
        indexes,
    }
}
