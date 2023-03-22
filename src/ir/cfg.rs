use crate::{analyses, checkers, ir, loaders};
use analyses::{run_worklist, SwitchAnalyzer};
use checkers::resolve_jumps;
use ir::lift_cfg;
use ir::types::*;
use loaders::types::VwModule;
use yaxpeax_core::analyses::control_flow::{get_cfg, VW_CFG};
use yaxpeax_core::arch::x86_64::MergedContextTable;

pub fn has_indirect_jumps(irmap: &IRMap) -> bool {
    for (_block_addr, ir_block) in irmap {
        for (_addr, ir_stmts) in ir_block {
            for (_idx, ir_stmt) in ir_stmts.iter().enumerate() {
                match ir_stmt {
                    Stmt::Branch(_, Value::Reg(_, _)) | Stmt::Branch(_, Value::Mem(_, _)) => {
                        return true
                    }
                    _ => (),
                }
            }
        }
    }
    false
}

pub fn fully_resolved_cfg(
    module: &VwModule,
    contexts: &MergedContextTable,
    addr: u64,
    strict: bool,
) -> (VW_CFG, IRMap) {
    unimplemented!()
    // let (cfg, _) = get_cfg(&module.program, contexts, addr, None);
    // let irmap = lift_cfg(module, &cfg, strict);
    // if !has_indirect_jumps(&irmap) {
    //     return (cfg, irmap);
    // }
    // return resolve_cfg(module, contexts, &cfg, &irmap, addr, strict);
}
