// use lucet_module::Module;
/*
use crate::utils::load_program;
use lucet_runtime::{self, DlModule, Limits, MmapRegion, PublicKey, Region, RunResult};
use lucet_module::{self,
    FunctionSpec, ModuleData, SerializedModule, TableElement, TrapManifest, TrapSite,
    VersionInfo, 
};
use lucet_runtime_internals::module::ModuleInternal;

use lucet_module::{
    FunctionHandle, FunctionIndex, FunctionPointer, Global, GlobalSpec, GlobalValue,
    HeapSpec, Signature, TrapCode,  ValueType, Module
};

// use lucet_module::{
//     FunctionSpec, Module, ModuleData, SerializedModule, TableElement, TrapManifest, TrapSite,
//     VersionInfo,
// };
use std::env;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::mem;
use object::{Object, ObjectSection, SymbolKind, SymbolScope};
use byteorder::{LittleEndian, ReadBytesExt};
use colored::Colorize;


#[derive(Debug)]
struct ArtifactSummary<'a> {
    buffer: &'a Vec<u8>,
    obj: &'a object::File<'a>,
    symbols: StandardSymbols<'a>,
    data_segments: Option<DataSegments>,
    serialized_module: Option<SerializedModule>,
    exported_functions: Vec<&'a str>,
    imported_symbols: Vec<&'a str>,
}

#[derive(Debug)]
struct StandardSymbols<'a> {
    lucet_module: Option<object::read::Symbol<'a>>,
}

#[derive(Debug)]
struct DataSegments {
    segments: Vec<DataSegment>,
}

#[derive(Debug)]
struct DataSegment {
    offset: u32,
    len: u32,
    data: Vec<u8>,
}


impl<'a> ArtifactSummary<'a> {
    fn new(buffer: &'a Vec<u8>, obj: &'a object::File<'_>) -> Self {
        Self {
            buffer: buffer,
            obj: obj,
            symbols: StandardSymbols { lucet_module: None },
            data_segments: None,
            serialized_module: None,
            exported_functions: Vec::new(),
            imported_symbols: Vec::new(),
        }
    }

    fn read_memory(&self, addr: u64, size: u64) -> Option<&'a [u8]> {
        // `addr` is really more of an offset from the start of the segment.
        for section in self.obj.sections() {
            let bytes = section.data_range(addr, size).ok().flatten();
            if bytes.is_some() {
                return bytes;
            }
        }

        None
    }

    fn gather(&mut self) {
        for sym in self.obj.symbols() {
            let sym = sym.1;
            match sym.name() {
                Some(ref name) if name == &"lucet_module" => {
                    self.symbols.lucet_module = Some(sym.clone())
                }
                Some(ref name) if name == &"" => continue,
                None => continue,
                _ => {
                    if sym.kind() == SymbolKind::Text && sym.scope() == SymbolScope::Dynamic {
                        self.exported_functions.push(sym.name().unwrap().into());
                    } else if sym.scope() == SymbolScope::Unknown {
                        self.imported_symbols.push(sym.name().unwrap().into());
                    }
                }
            }
        }

        self.serialized_module = self.symbols.lucet_module.as_ref().map(|module_sym| {
            let buffer = self
                .read_memory(
                    module_sym.address(),
                    mem::size_of::<SerializedModule>() as u64,
                )
                .unwrap();
            let mut rdr = Cursor::new(buffer);

            let version = VersionInfo::read_from(&mut rdr).unwrap();

            SerializedModule {
                version,
                module_data_ptr: rdr.read_u64::<LittleEndian>().unwrap(),
                module_data_len: rdr.read_u64::<LittleEndian>().unwrap(),
                tables_ptr: rdr.read_u64::<LittleEndian>().unwrap(),
                tables_len: rdr.read_u64::<LittleEndian>().unwrap(),
                function_manifest_ptr: rdr.read_u64::<LittleEndian>().unwrap(),
                function_manifest_len: rdr.read_u64::<LittleEndian>().unwrap(),
            }
        });
    }

    fn get_symbol_name_for_addr(&self, addr: u64) -> Option<&str> {
        self.obj
            .symbol_map()
            .get(addr)
            .map(|sym| sym.name().unwrap_or("(no name)"))
    }
}



// pub fn load_metadata(path : String){
//     // let module = DlModule::load(path).unwrap();

//     // // let module = DlModule::load(path).expect("module can be loaded");
//     // let min_globals_size = module.initial_globals_size();
//     // let globals_size = ((min_globals_size + 4096 - 1) / 4096) * 4096;
//     // println!("{0}", globals_size);

//     // let functions = module.function_manifest();
//     // for function in functions.iter(){
//     //     println!("{:?}", function.ptr());
//     // }
//     println!("Loading metadata");
//     let mut fd = File::open(path).expect("open");
//     let mut buffer = Vec::new();
//     fd.read_to_end(&mut buffer).expect("read");
//     let object = object::File::parse(&buffer).expect("parse");
//     println!("Summarizing Metadata");
//     let mut summary = ArtifactSummary::new(&buffer, &object);
//     println!("Gathering Summary");
//     summary.gather();
//     println!("Printing Summary");
//     print_summary(summary);
// }





fn print_summary(summary: ArtifactSummary<'_>) {
    println!("Required Symbols:");
    println!(
        "  {:30}: {}",
        "lucet_module",
        exists_to_str(&summary.symbols.lucet_module)
    );
    if let Some(ref serialized_module) = summary.serialized_module {
        println!("Native module components:");
        println!(
            "  {:30}: {}",
            "module_data_ptr",
            ptr_to_str(serialized_module.module_data_ptr)
        );
        println!(
            "  {:30}: {}",
            "module_data_len", serialized_module.module_data_len
        );
        println!(
            "  {:30}: {}",
            "tables_ptr",
            ptr_to_str(serialized_module.tables_ptr)
        );
        println!("  {:30}: {}", "tables_len", serialized_module.tables_len);
        println!(
            "  {:30}: {}",
            "function_manifest_ptr",
            ptr_to_str(serialized_module.function_manifest_ptr)
        );
        println!(
            "  {:30}: {}",
            "function_manifest_len", serialized_module.function_manifest_len
        );

        let tables_bytes = summary
            .read_memory(
                serialized_module.tables_ptr,
                serialized_module.tables_len * mem::size_of::<&[TableElement]>() as u64,
            )
            .unwrap();
        let tables = unsafe {
            std::slice::from_raw_parts(
                tables_bytes.as_ptr() as *const &[TableElement],
                serialized_module.tables_len as usize,
            )
        };
        let mut reconstructed_tables = Vec::new();
        // same situation as trap tables - these slices are valid as if the module was
        // dlopen'd, but we just read it as a flat file. So read through the ELF view and use
        // pointers to that for the real slices.

        for table in tables {
            let table_bytes = summary
                .read_memory(
                    table.as_ptr() as usize as u64,
                    (table.len() * mem::size_of::<TableElement>()) as u64,
                )
                .unwrap();
            reconstructed_tables.push(unsafe {
                std::slice::from_raw_parts(
                    table_bytes.as_ptr() as *const TableElement,
                    table.len() as usize,
                )
            });
        }

        let module = load_module(&summary, serialized_module, &reconstructed_tables);
        println!("\nModule:");
        // println!("{:?}", module.initial_globals_size());
        // summarize_module(&summary, &module);
    } else {
        println!("The symbol `lucet_module` is {}, so lucet-objdump cannot look at most of the interesting parts.", "MISSING".red().bold());
    }

    println!("");
    println!("Data Segments:");
    if let Some(data_segments) = summary.data_segments {
        println!("  {:6}: {}", "Count", data_segments.segments.len());
        for segment in &data_segments.segments {
            println!(
                "  {:7}: {:6}  {:6}: {:6}",
                "Offset", segment.offset, "Length", segment.len,
            );
        }
    } else {
        println!("  {}", "MISSING!".red().bold());
    }
}

fn ptr_to_str(p: u64) -> colored::ColoredString {
    if p != 0 {
        format!("exists; address: {:#x}", p).green()
    } else {
        "MISSING!".red().bold()
    }
}

fn exists_to_str<T>(p: &Option<T>) -> colored::ColoredString {
    return match p {
        Some(_) => "exists".green(),
        None => "MISSING!".red().bold(),
    };
}

fn load_module<'b, 'a: 'b>(
    summary: &'a ArtifactSummary<'a>,
    serialized_module: &SerializedModule,
    tables: &'b [&[TableElement]],
) -> Module<'b> {
    let module_data_bytes = summary
        .read_memory(
            serialized_module.module_data_ptr,
            serialized_module.module_data_len,
        )
        .unwrap();

    let module_data =
        ModuleData::deserialize(module_data_bytes).expect("ModuleData can be deserialized");

    let function_manifest_bytes = summary
        .read_memory(
            serialized_module.function_manifest_ptr,
            serialized_module.function_manifest_len,
        )
        .unwrap();
    let function_manifest = unsafe {
        std::slice::from_raw_parts(
            function_manifest_bytes.as_ptr() as *const FunctionSpec,
            serialized_module.function_manifest_len as usize,
        )
    };
    Module {
        version: serialized_module.version.clone(),
        module_data,
        tables,
        function_manifest,
    }
}

*/
