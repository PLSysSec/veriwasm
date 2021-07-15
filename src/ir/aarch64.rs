use crate::ir::types::{Stmt, IRMap};
use crate::VwMetadata;
use yaxpeax_core::analyses::control_flow::VW_CFG;
use crate::VwModule;
use capstone::prelude::*;
use crate::ir::types::ParseErr;

pub fn lift(
    instr: &Aarch64Insn,
    addr: &u64,
    metadata: &VwMetadata,
    strict: bool,
) -> Vec<Stmt> {
    unimplemented!()
}

fn parse_instr<'a>(
    instrs: BlockInstrs<'a>,
    metadata: &VwMetadata,
    strict: bool,
) -> IResult<'a, StmtResult> {
    if let Some((instr, rest)) = instrs.split_first() {
        let addr = instr.address();
        Ok((rest, (addr, lift(instr, &addr, metadata, strict))))
    } else {
        Err(ParseErr::Incomplete)
    }
}

fn parse_instrs<'a>(
    instrs: BlockInstrs,
    metadata: &VwMetadata,
    strict: bool,
) -> Vec<(Addr, Vec<Stmt>)> {
    let mut block_ir: Vec<(Addr, Vec<Stmt>)> = Vec::new();
    let mut rest = instrs;
    while let Ok((more, (addr, stmts))) = parse_instr(rest, metadata, strict) {
        rest = more;
        // if stmts.len() == 1 {
        //     if let Stmt::Branch(Opcode::JMP, _) = stmts[0] {
        //         // Don't continue past an unconditional jump --
        //         // Cranelift's new backend embeds constants in the
        //         // code stream at points (e.g. jump tables) and we
        //         // should not disassemble them as code.
        //         block_ir.push((addr, stmts));
        //         break;
        //     }
        // }
        block_ir.push((addr, stmts));
    }
    block_ir
}

fn disas_aarch64<'a>(cs: &'a Capstone, buf: &[u8], addr: u64) -> capstone::Instructions<'a>{
    match cs.disasm_all(buf, addr) {
        Ok(insns) => { 
            insns
        }
        Err(err) => {
            panic!();
        }
    }
}

pub fn lift_cfg(module: &VwModule, cfg: &VW_CFG, strict: bool) -> IRMap {
    let mut irmap = IRMap::new();
    let g = &cfg.graph;
    let cs = Capstone::new()
        .arm64()
        .build()
        .expect("Failed to create capstone handle");
    let text_segment = module.program.segments.iter().find(|seg| seg.name == ".text").expect("No text section?");

    for block_addr in g.nodes() {
        let block = cfg.get_block(block_addr);
        let buf = &text_segment.data[block.start as usize..block.end as usize];
        // let instrs_vec: Vec<Aarch64Insn> = disas_aarch64(&cs, buf, block.start).iter().collect();
        // let instrs = instrs_vec.as_slice();
        let instrs = disas_aarch64(&cs, buf, block.start);
        let block_ir = parse_instrs(&instrs, &module.metadata, strict);
        irmap.insert(block_addr, block_ir);
    }
    irmap
}

type IResult<'a, O> = Result<(BlockInstrs<'a>, O), ParseErr<BlockInstrs<'a>>>;
type BlockInstrs<'a> = &'a [Aarch64Insn<'a>];
type Addr = u64;
type StmtResult = (Addr, Vec<Stmt>);
type Aarch64Insn<'a> = capstone::Insn<'a>;
