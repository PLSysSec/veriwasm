use crate::ir::types::{Stmt, IRMap};
use crate::VwMetadata;
use yaxpeax_core::analyses::control_flow::VW_CFG;
use crate::VwModule;

pub fn lift(
    instr: &yaxpeax_x86::long_mode::Instruction,
    addr: &u64,
    metadata: &VwMetadata,
    strict: bool,
) -> Vec<Stmt> {
    unimplemented!()
}


pub fn lift_cfg(module: &VwModule, cfg: &VW_CFG, strict: bool) -> IRMap {
    let mut irmap = IRMap::new();
    let g = &cfg.graph;
    unimplemented!();
    // for block_addr in g.nodes() {
    //     let block = cfg.get_block(block_addr);

    //     let instrs_vec: Vec<(u64, X64Instruction)> = module
    //         .program
    //         .instructions_spanning(<AMD64 as Arch>::Decoder::default(), block.start, block.end)
    //         .collect();
    //     let instrs = instrs_vec.as_slice();
    //     let block_ir = parse_instrs(instrs, &module.metadata, strict);

    //     irmap.insert(block_addr, block_ir);
    // }
    // irmap
}