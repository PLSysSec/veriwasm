use crate::ir::types::{Stmt, IRMap};
use crate::VwMetadata;
use yaxpeax_core::analyses::control_flow::VW_CFG;
use crate::VwModule;
use capstone::prelude::*;
use capstone::arch::arm64;
use crate::ir::types::ParseErr;
use crate::ir::types::Value;
use capstone::arch::arm64::Arm64Operand;
use capstone::arch::ArchOperand;

fn reg_names<T, I>(cs: &Capstone, regs: T) -> String 
where T: Iterator<Item = I>,
I: Into<RegId>
{
    let names: Vec<String> = regs.map(|x| cs.reg_name(x.into()).unwrap()).collect();
    names.join(", ")
}

// Captures all register and flag sources
// TODO: Memory?
fn get_sources(cs: &Capstone, instr: &Aarch64Insn, detail: &InsnDetail) -> Vec<Value> {
    // let regs_read = detail.regs_read();
    // println!("regs read: {}", reg_names(cs, regs_read));
    // let regs_written = detail.regs_write();
    // println!("regs written: {}", reg_names(cs, regs_written));


    // let arch_detail = detail.arch_detail();
    // let operands = arch_detail.operands();
    // println!("{:?}", instr);
    // for op in operands{
    //     // let op: Arm64Operand = op.into();
    //     match op {
    //         ArchOperand::Arm64Operand(inner) => { println!("{:?} {:?}", inner.op_type, inner.access ); } 
    //         _ => panic!("Not aarch64?") 
    //     }
        
    // }
    unimplemented!();

    // let uses_vec = <AMD64 as ValueLocations>::decompose(instr);
    // let mut sources = Vec::new();
    // for (loc, dir) in uses_vec {
    //     match (loc, dir) {
    //         (Some(Location::Register(reg)), Direction::Read) => {
    //             sources.push(convert_reg(reg));
    //         }
    //         (Some(Location::ZF), Direction::Read) => {
    //             sources.push(Value::Reg(Zf, Size8));
    //         }
    //         (Some(Location::CF), Direction::Read) => {
    //             sources.push(Value::Reg(Cf, Size8));
    //         }
    //         (Some(Location::UnevalMem(op)), Direction::Read) => {
    //             sources.push(convert_operand(op, Size32)); // is Size32 right?
    //         }
    //         _ => {}
    //     }
    // }
    // return sources;
}

fn generic_clear(cs: &Capstone, instr: &Aarch64Insn) -> Vec<Stmt> {
    let mut stmts: Vec<Stmt> = vec![];
    let detail = cs.insn_detail(instr).expect("Unable to get detail");
    let sources = get_sources(cs, instr, &detail);
    unimplemented!();
    // let dsts = get_destinations(&instr);
    // for dst in dsts {
    //     stmts.push(Stmt::Clear(dst, sources.clone()));
    // }
    // stmts
}

pub fn lift(
    cs: &Capstone,
    instr: &Aarch64Insn,
    addr: &u64,
    metadata: &VwMetadata,
    strict: bool,
) -> Vec<Stmt> {
    let mut instrs = Vec::new();
    match instr.mnemonic(){
        other_insn => {
            if strict{
                println!("Unknown instruction: {:?}", other_insn);
                unimplemented!();
            } else {
                instrs.extend(generic_clear(cs, instr));
            }
        },
    }
    instrs
}

fn parse_instr<'a>(
    cs: &Capstone,
    instrs: BlockInstrs<'a>,
    metadata: &VwMetadata,
    strict: bool,
) -> IResult<'a, StmtResult> {
    if let Some((instr, rest)) = instrs.split_first() {
        let addr = instr.address();
        Ok((rest, (addr, lift(cs, instr, &addr, metadata, strict))))
    } else {
        Err(ParseErr::Incomplete)
    }
}

fn parse_instrs<'a>(
    cs: &Capstone,
    instrs: BlockInstrs,
    metadata: &VwMetadata,
    strict: bool,
) -> Vec<(Addr, Vec<Stmt>)> {
    let mut block_ir: Vec<(Addr, Vec<Stmt>)> = Vec::new();
    let mut rest = instrs;
    while let Ok((more, (addr, stmts))) = parse_instr(cs, rest, metadata, strict) {
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
    let mut cs = Capstone::new()
        .arm64()
        .mode(arm64::ArchMode::Arm)
        .build()
        .expect("Failed to create capstone handle");
    cs.set_detail(true).unwrap();
    let text_segment = module.program.segments.iter().find(|seg| seg.name == ".text").expect("No text section?");

    for block_addr in g.nodes() {
        let block = cfg.get_block(block_addr);
        let buf = &text_segment.data[block.start as usize..block.end as usize];
        let instrs = disas_aarch64(&cs, buf, block.start);
        let block_ir = parse_instrs(&cs, &instrs, &module.metadata, strict);
        irmap.insert(block_addr, block_ir);
    }
    irmap
}

type IResult<'a, O> = Result<(BlockInstrs<'a>, O), ParseErr<BlockInstrs<'a>>>;
type BlockInstrs<'a> = &'a [Aarch64Insn<'a>];
type Addr = u64;
type StmtResult = (Addr, Vec<Stmt>);
type Aarch64Insn<'a> = capstone::Insn<'a>;
