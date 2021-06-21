pub mod lucet;
pub mod utils;
pub mod wasmtime;
use crate::loaders::lucet::{load_lucet_metadata, load_lucet_program};
use crate::loaders::utils::LucetMetadata;
use crate::loaders::wasmtime::{load_wasmtime_metadata, load_wasmtime_program};
use core::str::FromStr;
use std::string::ParseError;
use yaxpeax_core::memory::repr::process::ModuleData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableType {
    Lucet,
    Wasmtime,
}

pub trait Loadable {
    fn load_program(&self, binpath: &str) -> ModuleData;
    fn load_metadata(&self, program: &ModuleData) -> LucetMetadata;
}

impl Loadable for ExecutableType {
    fn load_program(&self, binpath: &str) -> ModuleData {
        match self {
            ExecutableType::Lucet => load_lucet_program(binpath),
            ExecutableType::Wasmtime => load_wasmtime_program(binpath),
        }
    }

    fn load_metadata(&self, program: &ModuleData) -> LucetMetadata {
        match self {
            ExecutableType::Lucet => load_lucet_metadata(program),
            ExecutableType::Wasmtime => load_wasmtime_metadata(program),
        }
    }
}

impl FromStr for ExecutableType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_string().to_lowercase()[..] {
            "lucet" => Ok(ExecutableType::Lucet),
            "wasmtime" => Ok(ExecutableType::Wasmtime),
            _ => Err("Unknown executable type"),
        }
    }
}
