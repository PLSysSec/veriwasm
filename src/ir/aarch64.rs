use crate::ir::types::Stmt;
use crate::VW_Metadata;

pub fn lift(
    instr: &yaxpeax_x86::long_mode::Instruction,
    addr: &u64,
    metadata: &VW_Metadata,
) -> Vec<Stmt> {
    unimplemented!()
}
