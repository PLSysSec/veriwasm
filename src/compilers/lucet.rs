use yaxpeax_core::memory::reader;
use yaxpeax_core::memory::repr::process::{
    ELFExport, ELFImport, ELFSymbol, ModuleData, ModuleInfo,
};
use yaxpeax_core::memory::repr::FileRepr;

// pub fn load_wasmtime_program(binpath: &str) -> ModuleData {
//     let program = reader::load_from_path(Path::new(binpath)).unwrap();
//     let program = if let FileRepr::Executable(program) = program {
//         program
//     } else {
//         panic!("function:{} is not a valid path", binpath);
//     };
//     program
// }
