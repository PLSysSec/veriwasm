
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
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    X29,
    X30,
    X31,
    Nzcv,
    // Nf,
    // Zf,
    // Cf,
    // Vf,
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    D10,
    D11,
    D12,
    D13,
    D14,
    D15,
    D16,
    D17,
    D18,
    D19,
    D20,
    D21,
    D22,
    D23,
    D24,
    D25,
    D26,
    D27,
    D28,
    D29,
    D30,
    D31,
}

use self::Aarch64Regs::*;

impl Aarch64Regs {
    pub fn is_flag(self) -> bool {
        match self {
            Nzcv => true,
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
            0 => Ok(X0),
            1 => Ok(X1),
            2 => Ok(X2),
            3 => Ok(X3),
            4 => Ok(X4),
            5 => Ok(X5),
            6 => Ok(X6),
            7 => Ok(X7),
            8 => Ok(X8),
            9 => Ok(X9),
            10 => Ok(X10),
            11 => Ok(X11),
            12 => Ok(X12),
            13 => Ok(X13),
            14 => Ok(X14),
            15 => Ok(X15),
            16 => Ok(X16),
            17 => Ok(X17),
            18 => Ok(X18),
            19 => Ok(X19),
            20 => Ok(X20),
            21 => Ok(X21),
            22 => Ok(X22),
            23 => Ok(X23),
            24 => Ok(X24),
            25 => Ok(X25),
            26 => Ok(X26),
            27 => Ok(X27),
            28 => Ok(X28),
            29 => Ok(X29),
            30 => Ok(X30),
            31 => Ok(X31),
            32 => Ok(Nzcv),
            // 32 => Ok(Nf),
            // 33 => Ok(Zf),
            // 34 => Ok(Cf),
            // 35 => Ok(Vf),
            33 => Ok(D0), 
	        34 => Ok(D1),
	        35 => Ok(D2),
	        36 => Ok(D3),
	        37 => Ok(D4),
	        38 => Ok(D5),
	        39 => Ok(D6),
	        40 => Ok(D7),
	        41 => Ok(D8),
	        42 => Ok(D9),
	        43 => Ok(D10),
	        44 => Ok(D11),
	        45 => Ok(D12),
	        46 => Ok(D13),
	        47 => Ok(D14),
	        48 => Ok(D15),
	        49 => Ok(D16),
	        50 => Ok(D17),
	        51 => Ok(D18),
	        52 => Ok(D19),
	        53 => Ok(D20),
	        54 => Ok(D21),
	        55 => Ok(D22),
	        56 => Ok(D23),
	        57 => Ok(D24),
	        58 => Ok(D25),
	        59 => Ok(D26),
	        60 => Ok(D27),
	        61 => Ok(D28),
	        62 => Ok(D29),
	        63 => Ok(D30),
	        64 => Ok(D31),
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

// "ffr",
// "fp",
// "lr",
// "nzcv",
// "sp",
// "wsp",
// "wzr",
// "xzr",

fn convert_reg(reg: capstone::RegId) -> Value<Aarch64Regs> {
    match reg.0 {
        2 => Value::Reg(X29, Size64), // fp
        3 => Value::Reg(X30, Size64), // lr
        4 => Value::Reg(Nzcv, Size64), // NZCV
        5 => Value::Reg(X31, Size64), // sp
        r @ 9..=40 => Value::Reg(Aarch64Regs::try_from(r - 9).unwrap(), Size8),
        r @ 41..=72 => Value::Reg(Aarch64Regs::try_from(r - 41 + 33).unwrap(), Size64),
        r @ 73..=104 => Value::Reg(Aarch64Regs::try_from(r - 73).unwrap(), Size16),
        r @ 153..=184 => Value::Reg(Aarch64Regs::try_from(r - 153 + 33).unwrap(), Size32),
        r @ 185..=215 =>  Value::Reg(Aarch64Regs::try_from(r - 185).unwrap(), Size32),
        r @ 216..=244 => Value::Reg(Aarch64Regs::try_from(r - 216).unwrap(), Size32),
        _ => panic!("Unknown register: {:?}", reg),
    }
}

//TODO: handle all memory reads and writes
fn generic_clear(cs: &Capstone, instr: &Aarch64Insn) -> Vec<Stmt<Aarch64Regs>> {
    let mut stmts: Vec<Stmt<Aarch64Regs>> = vec![];
    let detail = cs.insn_detail(instr).expect("Unable to get detail");
    // let sources = get_sources(cs, instr, &detail);
    let regs_read = detail.regs_read();
    println!("capstone reg sources: {:?}", reg_names(cs, regs_read.clone()));
    let reg_sources: Vec<Value<Aarch64Regs>> = regs_read.map(|reg| convert_reg(reg)).collect();
    println!("VW reg sources: {:?}", reg_sources);
    let regs_write = detail.regs_write();
    println!("capstone reg sources: {:?}", reg_names(cs, regs_write.clone()));
    let reg_dsts: Vec<Value<Aarch64Regs>> = regs_write.map(|reg| convert_reg(reg)).collect();
    println!("VW reg sources: {:?}", reg_dsts);
    for dst in reg_dsts {
        stmts.push(Stmt::Clear(dst, reg_sources.clone()));
    }
    stmts
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
        self == &X31
    }

    fn is_rbp(&self) -> bool {
        self == &X29
    }

    fn is_zf(&self) -> bool {
        self == &Nzcv
    }

    fn pinned_heap_reg() -> Self {
        unimplemented!()
    }
}
