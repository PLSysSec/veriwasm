use crate::{loaders, runner};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use elfkit::relocation::RelocationType;
use elfkit::{symbol, types, DynamicContent, Elf, SectionContent};
use goblin::Object;
use loaders::types::{VwFuncInfo, VwMetadata, VwModule};
use loaders::utils::*;
use loaders::utils::{deconstruct_elf, get_symbol_addr};
use lucet_module;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Cursor;
use std::io::{Read, Seek, SeekFrom};
use std::mem;
use std::path::Path;
use std::string::String;
use yaxpeax_core::memory::repr::process::{ModuleData, Segment};
use yaxpeax_core::memory::repr::FileRepr;

pub fn lucet_get_plt_funcs(binpath: &str) -> Vec<(u64, String)> {
    //Extract relocation symbols
    let mut in_file = OpenOptions::new().read(true).open(binpath).unwrap();
    let mut elf = Elf::from_reader(&mut in_file).unwrap();
    elf.load_all().unwrap();
    // Parse relocs to get mapping from target to name
    let mut target_to_name = HashMap::new();
    for section in &elf.sections {
        match section.content {
            SectionContent::Relocations(ref relocs) => {
                for reloc in relocs {
                    elf.sections
                        .get(section.header.link as usize)
                        .and_then(|sec| sec.content.as_symbols())
                        .and_then(|symbols| symbols.get(reloc.sym as usize))
                        .and_then(|symbol| {
                            if symbol.name.len() > 0 {
                                target_to_name.insert(reloc.addr, symbol.name.clone());
                                // println!("{:x} {:} ", reloc.addr, symbol.name);
                                Some(())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| {
                            // print!("{: <20.20} ", reloc.sym);
                        });
                }
            }
            _ => (),
        }
    }
    // Parse PLT to get mapping from address to target
    let mut addr_to_target = HashMap::new();
    let plt_section = elf.sections.iter().find(|sec| sec.name == ".plt");
    // if plt_section.is_none(){
    //     return Vec::new();
    // };
    let plt_section = plt_section.unwrap();
    let plt_start = plt_section.header.addr;
    if let SectionContent::Raw(buf) = &plt_section.content {
        let mut rdr = Cursor::new(buf);
        let mut idx: usize = 0;
        while idx < buf.len() {
            rdr.seek(SeekFrom::Current(2)).unwrap();
            let offset = rdr.read_i32::<LittleEndian>().unwrap();
            rdr.seek(SeekFrom::Current(10)).unwrap();
            let addr = plt_start + (idx as u64);
            println!("{:x} {:x} {:x}", plt_start, idx, offset);
            let target = plt_start + (idx as u64) + (offset as u64) + 6;
            addr_to_target.insert(addr, target);
            idx += 16;
        }
    } else {
        panic!("No plt section?");
    }
    // println!("===== Addr to Target =======");
    // for (addr,target) in &addr_to_target{
    //     println!("{:x} {:x}", addr, target);
    // }
    // println!("===== Target to Name =======");
    // for (target, name) in &target_to_name{
    //     println!("{:x} {:}", target, name);
    // }

    let mut plt_funcs: Vec<(u64, String)> = Vec::new();
    for (addr, target) in &addr_to_target {
        if target_to_name.contains_key(target) {
            let name = target_to_name[target].clone();
            plt_funcs.push((*addr, name));
        }
    }
    // println!("plt_funcs: {:?}", plt_funcs);

    // unimplemented!();
    plt_funcs
}

// pub fn load_lucet_program(binpath: &str) -> ModuleData {
//     let program = yaxpeax_core::memory::reader::load_from_path(Path::new(binpath)).unwrap();
//     if let FileRepr::Executable(program) = program {
//         program
//     } else {
//         panic!("function:{} is not a valid path", binpath)
//     }
// }

// pub fn load_lucet_program(config: &runner::Config) -> VwModule {
//     let program =
//         yaxpeax_core::memory::reader::load_from_path(Path::new(&config.module_path)).unwrap();
//     if let FileRepr::Executable(program) = program {
//         let metadata = load_lucet_metadata(&program);
//         VwModule {
//             program,
//             metadata,
//             format: config.executable_type,
//             arch: config.arch,
//         }
//     } else {
//         panic!("function:{} is not a valid path", config.module_path)
//     }
// }

pub fn load_lucet_metadata(program: &ModuleData) -> VwMetadata {
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

pub fn load_lucet_program(config: &runner::Config) -> VwModule {
    let program =
        yaxpeax_core::memory::reader::load_from_path(Path::new(&config.module_path)).unwrap();
    if let FileRepr::Executable(program) = program {
        let metadata = load_lucet_metadata(&program);
        VwModule {
            program,
            metadata,
            format: config.executable_type,
            arch: config.arch,
        }
    } else {
        panic!("function:{} is not a valid path", config.module_path)
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
