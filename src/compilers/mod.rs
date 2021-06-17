pub mod wasmtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetCompiler {
    Lucet,
    Wasmtime,
}
