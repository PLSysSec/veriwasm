pub mod lucet;
pub mod wasmtime;
use core::str::FromStr;
use std::string::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetCompiler {
    Lucet,
    Wasmtime,
}

impl FromStr for TargetCompiler {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_string().to_lowercase()[..] {
            "lucet" => Ok(TargetCompiler::Lucet),
            "wasmtime" => Ok(TargetCompiler::Wasmtime),
            _ => Err("Unknown compiler"),
        }
    }
}
