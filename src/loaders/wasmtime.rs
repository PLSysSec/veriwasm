use crate::runner::Config;
use crate::{loaders, runner};
use loaders::types::{VwFuncInfo, VwMetadata, VwModule};
use loaders::utils::deconstruct_elf;
use loaders::utils::*;
use std::fs;
use wasmtime::*;
use yaxpeax_core::goblin::Object;
use yaxpeax_core::memory::repr::process::ModuleData;
use yaxpeax_core::memory::repr::process::Segment;

//yaxpeax doesnt load .o files correctly, so this code
// manually adds memory regions corresponding to ELF sections
// (yaxpeax does this by segments, but .o files may not have segments)
fn fixup_object_file(program: &mut ModuleData, obj: &[u8]) {
    // let elf = program.module_info().unwrap();
    let elf = match Object::parse(obj) {
        Ok(obj @ Object::Elf(_)) => match obj {
            Object::Elf(elf) => elf,
            _ => panic!(),
        },
        _ => panic!(),
    };

    for section in elf.section_headers.iter() {
        if section.sh_name == 0 {
            continue;
        }
        //Load data for section
        let mut section_data = vec![0; section.sh_size as usize];
        for idx in 0..section.sh_size {
            section_data[idx as usize] = obj[(section.sh_offset + idx) as usize];
        }
        //add as segment
        let new_section = Segment {
            start: section.sh_addr as usize, // virtual addr
            data: section_data,
            name: elf
                .shdr_strtab
                .get(section.sh_name)
                .unwrap()
                .unwrap()
                .to_string(),
        };
        program.segments.push(new_section);
    }
}

fn load_wasmtime_metadata(program: &ModuleData) -> VwMetadata {
    let (_, sections, entrypoint, imports, exports, symbols) = deconstruct_elf(program);

    // unimplemented!();

    // let guest_table_0 = get_symbol_addr(symbols, "guest_table_0").unwrap();
    // let lucet_tables = get_symbol_addr(symbols, "lucet_tables").unwrap();
    // let lucet_probestack = get_symbol_addr(symbols, "lucet_probestack").unwrap();
    // println!(
    //     "guest_table_0 = {:x} lucet_tables = {:x} probestack = {:x}",
    //     guest_table_0, lucet_tables, lucet_probestack
    // );
    VwMetadata {
        guest_table_0: 0,
        lucet_tables: 0,
        lucet_probestack: 0,
    }
}

pub fn load_wasmtime_program(config: &runner::Config) -> VwModule {
    let path = &config.module_path;
    let buffer = fs::read(path).expect("Something went wrong reading the file");
    let store: Store = Store::default();
    // Deserialize wasmtime module
    let module = unsafe { Module::deserialize(store.engine(), &buffer).unwrap() }; 
    unimplemented!();
    // let obj = module.obj();

    // match ModuleData::load_from(&obj, path.to_string()) {
    //     Some(mut program) => {
    //         fixup_object_file(&mut program, &obj);
    //         let metadata = load_wasmtime_metadata(&program);
    //         VwModule {
    //             program,
    //             metadata,
    //             format: config.executable_type,
    //             arch: config.arch,
    //         }
    //     } //{ FileRepr::Executable(data) }
    //     None => {
    //         panic!("function:{} is not a valid path", path)
    //     }
    // }
}

// We do not need to check handwritten trampoline functions
pub fn is_valid_wasmtime_func_name(name: &String) -> bool {
    // true
    !name.starts_with("_trampoline")
}

pub fn get_wasmtime_func_signatures(program: &ModuleData) -> VwFuncInfo {
    unimplemented!();
}

pub fn wasmtime_get_plt_funcs(binpath: &str) -> Vec<(u64, String)> {
    unimplemented!();
}
