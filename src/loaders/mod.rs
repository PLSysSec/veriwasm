mod lucet;
pub mod types;
pub mod utils;
mod wasmtime;
use crate::loaders;
use crate::runner::Config;
use loaders::lucet::*;
use loaders::types::*;
use loaders::wasmtime::*;
use yaxpeax_core::memory::repr::process::ModuleData;

// TODO: this should be static dispatch, not dynamic dispatch
// not performance critical, but static dispatch is more rusty

pub fn load_program(config: &Config) -> VwModule {
    match config.executable_type {
        ExecutableType::Lucet => load_lucet_program(config),
        ExecutableType::Wasmtime => load_wasmtime_program(config),
    }
}

pub trait Loadable {
    fn is_valid_func_name(&self, name: &String) -> bool;
    fn get_func_signatures(&self, program: &ModuleData) -> VwFuncInfo;
    fn get_plt_funcs(&self, binpath: &str) -> Vec<(u64, String)>;
}

impl Loadable for ExecutableType {
    fn is_valid_func_name(&self, name: &String) -> bool {
        match self {
            ExecutableType::Lucet => is_valid_lucet_func_name(name),
            ExecutableType::Wasmtime => is_valid_wasmtime_func_name(name),
        }
    }

    fn get_func_signatures(&self, program: &ModuleData) -> VwFuncInfo {
        match self {
            ExecutableType::Lucet => get_lucet_func_signatures(program),
            ExecutableType::Wasmtime => get_wasmtime_func_signatures(program),
        }
    }

    fn get_plt_funcs(&self, binpath: &str) -> Vec<(u64, String)> {
        match self {
            ExecutableType::Lucet => lucet_get_plt_funcs(binpath),
            ExecutableType::Wasmtime => wasmtime_get_plt_funcs(binpath),
        }
    }
}
