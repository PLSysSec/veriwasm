use std::path::Path;
use yaxpeax_core::memory::reader;
use yaxpeax_core::memory::repr::process::{
    ELFExport, ELFImport, ELFSymbol, ModuleData, ModuleInfo,
};
use yaxpeax_core::memory::repr::FileRepr;

// use wasmtime::*;
use std::env;
use std::fs;

pub fn load_wasmtime_program(binpath: &str) -> ModuleData {
    unimplemented!();

    let program = yaxpeax_core::memory::reader::load_from_path(Path::new(binpath)).unwrap();
    let program = if let FileRepr::Executable(program) = program {
        program
    } else {
        panic!("function:{} is not a valid path", binpath);
    };
    program
}

// fn deserialize_module(path: &String) -> Module {

//     let buffer = fs::read(path)
//     .expect("Something went wrong reading the file");

//     // Configure the initial compilation environment, creating the global
//     // `Store` structure. Note that you can also tweak configuration settings
//     // with a `Config` and an `Engine` if desired.
//     println!("Initializing...");
//     let mut store: Store<()> = Store::default();

//     // Compile the wasm binary into an in-memory instance of a `Module`. Note
//     // that this is `unsafe` because it is our responsibility for guaranteeing
//     // that these bytes are valid precompiled module bytes. We know that from
//     // the structure of this example program.
//     println!("Deserialize module...");
//     let module = unsafe { Module::deserialize(store.engine(), buffer).unwrap() };

//     // Next we poke around a bit to extract the `run` function from the module.
//     // println!("Extracting export...");
//     // let run = instance.get_typed_func::<(), (), _>(&mut store, "run")?;

//     // And last but not least we can call it!
//     // println!("Calling export...");
//     // run.call(&mut store, ())?;

//     println!("Done.");
//     module
// }

// fn main() {
//     println!("Hello, world!");
//     let args: Vec<String> = env::args().collect();

//     let filename = &args[1];
//     let module = deserialize_module(filename);
//     let imports = module.imports();

//     for import in imports{
//         println!("Import: {:?}", import);
//     }
//     // println!("Imports: {:?}", imports);
//     let exports = module.exports();
//     for export in exports{
//         println!("Export: {:?}", export);
//     }
//     // println!("Exports: {:?}", exports);
//     let obj = module.obj();//.artifacts.obj
//     println!("Ok, now I'm really done!");
// }
