use crate::ir::types::ParseErr;
use crate::ir::types::Value;
use crate::ir::types::{IRMap, Stmt, ValSize};
use crate::VwMetadata;
use crate::VwModule;
use capstone::arch::arm64;
use capstone::arch::arm64::Arm64Operand;
use capstone::arch::ArchOperand;
use capstone::prelude::*;
use core::convert::TryFrom;
use yaxpeax_core::analyses::control_flow::VW_CFG;
use ValSize::{Size128, Size16, Size256, Size32, Size512, Size64, Size8};

use crate::ir::types::RegT;

// TODO: this should not implement PartialOrd
// TODO: add flags iter
#[derive(PartialEq, PartialOrd, Clone, Eq, Debug, Copy, Hash)]
pub enum Aarch64Regs {
    W0,
    W1,
    W2,
    W3,
    W4,
    W5,
    W6,
    W7,
    W8,
    W9,
    W10,
    W11,
    W12,
    W13,
    W14,
    W15,
    W16,
    W17,
    W18,
    W19,
    W20,
    W21,
    W22,
    W23,
    W24,
    W25,
    W26,
    W27,
    W28,
    W29,
    W30,
    W31,
    Nf,
    Zf,
    Cf,
    Vf,
}

use self::Aarch64Regs::*;

impl Aarch64Regs {
    pub fn is_flag(self) -> bool {
        match self {
            Nf | Zf | Cf | Vf => true,
            _ => false,
        }
    }
}

// pub struct Aarch64RegsIterator {
//     current_reg: u16,
// }

// impl Aarch64Regs {
//     pub fn iter() -> Aarch64RegsIterator {
//         Aarch64RegsIterator {
//             current_reg: 0,
//         }
//     }
// }

// impl Iterator for Aarch64RegsIterator {
//     type Item = Aarch64Regs;

//     fn next(&mut self) -> Option<Self::Item> {
//         let next_reg = self.current_reg + 1;
//         match Aarch64Regs::try_from(next_reg) {
//             Ok(r) => {
//                 let current = self.current_reg;
//                 self.current_reg = next_reg;
//                 Some(r)
//             }
//             Err(_) => None
//         }
//     }
// }

impl TryFrom<u16> for Aarch64Regs {
    type Error = std::string::String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(W0),
            1 => Ok(W1),
            2 => Ok(W2),
            3 => Ok(W3),
            4 => Ok(W4),
            5 => Ok(W5),
            6 => Ok(W6),
            7 => Ok(W7),
            8 => Ok(W8),
            9 => Ok(W9),
            10 => Ok(W10),
            11 => Ok(W11),
            12 => Ok(W12),
            13 => Ok(W13),
            14 => Ok(W14),
            15 => Ok(W15),
            16 => Ok(W16),
            17 => Ok(W17),
            18 => Ok(W18),
            19 => Ok(W19),
            20 => Ok(W20),
            21 => Ok(W21),
            22 => Ok(W22),
            23 => Ok(W23),
            24 => Ok(W24),
            25 => Ok(W25),
            26 => Ok(W26),
            27 => Ok(W27),
            28 => Ok(W28),
            29 => Ok(W29),
            30 => Ok(W30),
            31 => Ok(W31),
            32 => Ok(Nf),
            33 => Ok(Zf),
            34 => Ok(Cf),
            35 => Ok(Vf),
            _ => Err(format!("Unknown register: index = {:?}", value)),
        }
    }
}

impl TryFrom<u8> for Aarch64Regs {
    type Error = std::string::String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_from(value as u16)
    }
}

impl From<Aarch64Regs> for u16 {
    fn from(value: Aarch64Regs) -> Self {
        value as u16
    }
}

impl From<Aarch64Regs> for u8 {
    fn from(value: Aarch64Regs) -> Self {
        value as u8
    }
}

fn reg_names<T, I>(cs: &Capstone, regs: T) -> String
where
    T: Iterator<Item = I>,
    I: Into<RegId>,
{
    let names: Vec<String> = regs.map(|x| cs.reg_name(x.into()).unwrap()).collect();
    names.join(", ")
}

// // Captures all register and flag sources
// // TODO: Memory?
// fn get_sources(cs: &Capstone, instr: &Aarch64Insn, detail: &InsnDetail) -> Vec<Value> {
//     let regs_read = detail.regs_read().iter().map(|&op| convert_op(op));
//     println!("{:?}", regs_read);
//     println!("regs read: {}", reg_names(cs, regs_read));
//     let regs_written = detail.regs_write().iter().map(|&op| convert_op(op));
//     println!("{:?}", regs_written);
//     println!("regs written: {}", reg_names(cs, regs_written));

//     let arch_detail = detail.arch_detail();
//     let operands = arch_detail.operands();
//     println!("{:?} {:?}", instr, arch_detail);
//     for op in operands{
//         // let op: Arm64Operand = op.into();
//         match op {
//             ArchOperand::Arm64Operand(inner) => { println!("{:?} {:?}", inner.op_type, inner.access ); }
//             _ => panic!("Not aarch64?")
//         }

//     }
//     unimplemented!();

//     // let uses_vec = <AMD64 as ValueLocations>::decompose(instr);
//     // let mut sources = Vec::new();
//     // for (loc, dir) in uses_vec {
//     //     match (loc, dir) {
//     //         (Some(Location::Register(reg)), Direction::Read) => {
//     //             sources.push(convert_reg(reg));
//     //         }
//     //         (Some(Location::ZF), Direction::Read) => {
//     //             sources.push(Value::Reg(Zf, Size8));
//     //         }
//     //         (Some(Location::CF), Direction::Read) => {
//     //             sources.push(Value::Reg(Cf, Size8));
//     //         }
//     //         (Some(Location::UnevalMem(op)), Direction::Read) => {
//     //             sources.push(convert_operand(op, Size32)); // is Size32 right?
//     //         }
//     //         _ => {}
//     //     }
//     // }
//     // return sources;
// }

fn convert_reg(op: capstone::RegId) -> Value<Aarch64Regs> {
    Value::Reg(Aarch64Regs::try_from(op.0).unwrap(), Size64)
}

fn generic_clear(cs: &Capstone, instr: &Aarch64Insn) -> Vec<Stmt<Aarch64Regs>> {
    let mut stmts: Vec<Stmt<Aarch64Regs>> = vec![];
    let detail = cs.insn_detail(instr).expect("Unable to get detail");
    // let sources = get_sources(cs, instr, &detail);
    let regs_read = detail.regs_read();
    println!("capstone reg sources: {:?}", regs_read);
    let reg_sources: Vec<Value<Aarch64Regs>> = regs_read.map(|reg| convert_reg(reg)).collect();
    println!("VW reg sources: {:?}", reg_sources);
    let regs_write = detail.regs_write();
    println!("capstone reg sources: {:?}", regs_write);
    let reg_dsts: Vec<Value<Aarch64Regs>> = regs_write.map(|reg| convert_reg(reg)).collect();
    println!("VW reg sources: {:?}", reg_dsts);
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
) -> Vec<Stmt<Aarch64Regs>> {
    let mut instrs = Vec::new();
    match instr.mnemonic() {
        other_insn => {
            if strict {
                println!("Unknown instruction: {:?}", other_insn);
                unimplemented!();
            } else {
                instrs.extend(generic_clear(cs, instr));
            }
        }
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
) -> Vec<(Addr, Vec<Stmt<Aarch64Regs>>)> {
    let mut block_ir: Vec<(Addr, Vec<Stmt<Aarch64Regs>>)> = Vec::new();
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

fn disas_aarch64<'a>(cs: &'a Capstone, buf: &[u8], addr: u64) -> capstone::Instructions<'a> {
    match cs.disasm_all(buf, addr) {
        Ok(insns) => insns,
        Err(err) => {
            panic!();
        }
    }
}

pub fn lift_cfg(module: &VwModule, cfg: &VW_CFG, strict: bool) -> IRMap<Aarch64Regs> {
    let mut irmap = IRMap::new();
    let g = &cfg.graph;
    let mut cs = Capstone::new()
        .arm64()
        .mode(arm64::ArchMode::Arm)
        .build()
        .expect("Failed to create capstone handle");
    cs.set_detail(true).unwrap();
    let text_segment = module
        .program
        .segments
        .iter()
        .find(|seg| seg.name == ".text")
        .expect("No text section?");

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
type StmtResult = (Addr, Vec<Stmt<Aarch64Regs>>);
type Aarch64Insn<'a> = capstone::Insn<'a>;

impl RegT for Aarch64Regs {
    fn is_rsp(&self) -> bool {
        self == &W31
    }

    fn is_rbp(&self) -> bool {
        self == &W29
    }

    fn is_zf(&self) -> bool {
        self == &Zf
    }

    fn pinned_heap_reg() -> Self {
        unimplemented!()
    }
}
