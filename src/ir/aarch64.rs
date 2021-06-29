use crate::ir::types::Stmt;
use crate::VwMetadata;

pub fn lift(
    instr: &yaxpeax_x86::long_mode::Instruction,
    addr: &u64,
    metadata: &VwMetadata,
    strict: bool,
) -> Vec<Stmt> {
    unimplemented!()
}
