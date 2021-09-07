use crate::ir::types::ParseErr;
use crate::ir::types::{
    Binopcode, IRBlock, IRMap, MemArg, MemArgs, Stmt, Unopcode, ValSize, Value,
};
use crate::VwMetadata;
use crate::VwModule;
use byteorder::{BigEndian, ReadBytesExt};
use capstone::arch::arm64;
use capstone::arch::arm64::{Arm64OpMem, Arm64Operand, Arm64OperandType};
use capstone::arch::ArchOperand;
use capstone::prelude::*;
use core::convert::TryFrom;
use yaxpeax_core::analyses::control_flow::{VW_Block, VW_CFG};
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

    Xz,
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
            65 => Ok(Xz),
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

fn convert_reg(reg: capstone::RegId) -> Value<Aarch64Regs> {
    match reg.0 {
        2 => Value::Reg(X29, Size64),  // fp
        3 => Value::Reg(X30, Size64),  // lr
        4 => Value::Reg(Nzcv, Size64), // NZCV
        5 => Value::Reg(X31, Size64),  // sp
        7 => Value::Reg(Xz, Size32),   // wzr
        8 => Value::Reg(Xz, Size64),   // xzr

        r @ 9..=40 => Value::Reg(Aarch64Regs::try_from(r - 9).unwrap(), Size8), // 8 bit registers
        r @ 41..=72 => Value::Reg(Aarch64Regs::try_from(r - 41 + 33).unwrap(), Size64), //  D registers
        r @ 73..=104 => Value::Reg(Aarch64Regs::try_from(r - 73).unwrap(), Size16), // H registers
        r @ 153..=184 => Value::Reg(Aarch64Regs::try_from(r - 153 + 33).unwrap(), Size32), // S registers
        r @ 185..=215 => Value::Reg(Aarch64Regs::try_from(r - 185).unwrap(), Size32), // W registers
        r @ 216..=244 => Value::Reg(Aarch64Regs::try_from(r - 216).unwrap(), Size64), // X registers
        r @ 277..=304 => Value::Reg(Aarch64Regs::try_from(r - 277 + 33).unwrap(), Size128), // V registers
        _ => panic!("Unknown register: {:?}", reg),
    }
}

//TODO: handle all memory reads and writes
fn generic_clear(cs: &Capstone, instr: &Aarch64Insn) -> Vec<Stmt<Aarch64Regs>> {
    let mut stmts: Vec<Stmt<Aarch64Regs>> = vec![];
    let detail = cs.insn_detail(instr).expect("Unable to get detail");
    // let sources = get_sources(cs, instr, &detail);
    let regs_read = detail.regs_read();
    // println!("capstone reg sources: {:?}", reg_names(cs, regs_read.clone()));
    let reg_sources: Vec<Value<Aarch64Regs>> = regs_read.map(|reg| convert_reg(reg)).collect();
    // println!("VW reg sources: {:?}", reg_sources);
    let regs_write = detail.regs_write();
    // println!("capstone reg dsts: {:?}", reg_names(cs, regs_write.clone()));
    let reg_dsts: Vec<Value<Aarch64Regs>> = regs_write.map(|reg| convert_reg(reg)).collect();
    // println!("VW reg dsts: {:?}", reg_dsts);
    for dst in reg_dsts {
        stmts.push(Stmt::Clear(dst, reg_sources.clone()));
    }
    stmts
}

fn convert_memargs(memargs: &Arm64OpMem, sz: &ValSize) -> Value<Aarch64Regs> {
    let imm = memargs.disp() as i64;
    let disp = MemArg::try_from(Value::from(imm)).unwrap();
    let memargs = match (memargs.base().0, memargs.index().0) {
        (0, 0) => MemArgs::Mem1Arg(disp),
        (b, 0) => {
            let base = MemArg::try_from(convert_reg(memargs.base())).unwrap();
            if imm == 0 {
                MemArgs::Mem1Arg(base)
            } else {
                MemArgs::Mem2Args(base, disp)
            }
        }
        (b, i) => {
            let base = MemArg::try_from(convert_reg(memargs.base())).unwrap();
            let index = MemArg::try_from(convert_reg(memargs.index())).unwrap();
            if imm == 0 {
                MemArgs::Mem2Args(base, index)
            } else {
                MemArgs::Mem3Args(base, index, disp)
            }
        }
    };
    Value::Mem(*sz, memargs)
}

fn writeback_pre_index(memargs: &Arm64OpMem) -> Stmt<Aarch64Regs> {
    let imm = memargs.disp() as i64;
    if imm == 0 {
        panic!("Writeback of 0: {:?}", memargs);
        //return None;
    }
    let disp = Value::from(imm);
    match (memargs.base().0, memargs.index().0) {
        (b, _) if b != 0 => {
            let base = convert_reg(memargs.base());
            return Stmt::Binop(Binopcode::Add, base.clone(), base.clone(), disp);
            // base += disp
        }
        _ => panic!("Wierd looking writeback: {:?}", memargs),
    };
}

fn writeback_post_index(memargs: &Arm64OpMem, v: &Value<Aarch64Regs>) -> Stmt<Aarch64Regs> {
    match (memargs.base().0, memargs.index().0) {
        (b, _) if b != 0 => {
            let base = convert_reg(memargs.base());
            return Stmt::Binop(Binopcode::Add, base.clone(), base.clone(), v.clone());
            // base += disp
        }
        _ => panic!("Wierd looking writeback: {:?}", memargs),
    };
}

fn convert_operand_mem(cs: &Capstone, op: &Arm64Operand, sz: &ValSize) -> Value<Aarch64Regs> {
    let memargs = convert_op_to_mem(op);
    convert_memargs(&memargs, sz)
}

fn convert_op_to_mem(op: &Arm64Operand) -> Arm64OpMem {
    match &op.op_type {
        Arm64OperandType::Mem(memargs) => memargs.clone(),
        other => panic!("Unknown Operand in conver_memargs: {:?}", other),
    }
}

fn convert_operand_no_mem(cs: &Capstone, op: &Arm64Operand) -> Value<Aarch64Regs> {
    match &op.op_type {
        Arm64OperandType::Reg(reg) => convert_reg(*reg),
        Arm64OperandType::Imm(imm) => Value::from(*imm),
        Arm64OperandType::Mem(memargs) => panic!(
            "convert_operand_no_mem called on mem: memargs = {:?}",
            memargs
        ),
        Arm64OperandType::Fp(f_imm) => panic!("unknnown operand: f_imm = {:?}", f_imm),
        Arm64OperandType::Cimm(c_imm) => panic!("unknnown operand: c_imm = {:?}", c_imm),
        other => panic!("Unknown Operand: {:?}", other),
    }
}

fn convert_operand_any(cs: &Capstone, op: &Arm64Operand, sz: &ValSize) -> Value<Aarch64Regs> {
    match &op.op_type {
        Arm64OperandType::Reg(reg) => convert_reg(*reg),
        Arm64OperandType::Imm(imm) => Value::from(*imm),
        Arm64OperandType::Mem(memargs) => convert_memargs(memargs, sz),
        Arm64OperandType::Fp(f_imm) => panic!("unknnown operand: f_imm = {:?}", f_imm),
        Arm64OperandType::Cimm(c_imm) => panic!("unknnown operand: c_imm = {:?}", c_imm),
        other => panic!("Unknown Operand: {:?}", other),
    }
}

fn get_aarch64_operands(cs: &Capstone, instr: &Aarch64Insn) -> Vec<Arm64Operand> {
    let detail = cs.insn_detail(instr).expect("Unable to get detail");
    let arch_detail = detail.arch_detail();
    let operands = arch_detail.operands();
    operands
        .iter()
        .map(|op| match op {
            ArchOperand::Arm64Operand(inner) => inner.clone(),
            _ => panic!("Not aarch64?"),
        })
        .collect()
}

fn unop(cs: &Capstone, opcode: Unopcode, instr: &Aarch64Insn) -> Stmt<Aarch64Regs> {
    let operands = get_aarch64_operands(cs, instr);
    assert_eq!(operands.len(), 2);
    let dst = convert_operand_no_mem(cs, &operands[0]);
    let src = convert_operand_no_mem(cs, &operands[1]);
    Stmt::Unop(opcode, dst, src)
}

fn parse_call(cs: &Capstone, instr: &Aarch64Insn) -> Stmt<Aarch64Regs> {
    let operands = get_aarch64_operands(cs, instr);
    assert_eq!(operands.len(), 1);
    let dst = convert_operand_no_mem(cs, &operands[0]);
    Stmt::Call(dst)
}

// fn do_load(){
//     let dst = convert_operand_no_mem(cs, &operands[0]);
//     let sz = dst.get_size();
//     let src = convert_operand_mem(cs, &operands[1], &sz);
//     Stmt::Unop(Unopcode::Mov, dst, src)
// }

fn parse_ldur(cs: &Capstone, instr: &Aarch64Insn) -> Stmt<Aarch64Regs> {
    let operands = get_aarch64_operands(cs, instr);
    assert_eq!(operands.len(), 2);
    let dst = convert_operand_no_mem(cs, &operands[0]);
    let sz = dst.get_size();
    let src = convert_operand_mem(cs, &operands[1], &sz);
    Stmt::Unop(Unopcode::Mov, dst, src)
}

fn parse_stur(cs: &Capstone, instr: &Aarch64Insn) -> Stmt<Aarch64Regs> {
    let operands = get_aarch64_operands(cs, instr);
    assert_eq!(operands.len(), 2);
    let src = convert_operand_no_mem(cs, &operands[0]);
    let sz = src.get_size();
    let dst = convert_operand_mem(cs, &operands[1], &sz);
    Stmt::Unop(Unopcode::Mov, dst, src)
}

// fn mov_to_mem(cs: &Capstone, op_dst: Arm64Operand, op_src: Arm64Operand){
//     let src = convert_operand_no_mem(cs, &operands[0]);
//     let sz = src.get_size();
//     let dst = convert_operand_mem(cs, &operands[1], &sz);
//     let stmt = Stmt::Unop(Unopcode::Mov, dst, src);
// }

//TODO: handle extenders

/// Parse the ldr (load register) instruction
/// This instruction has 4 forms: immediate, standard, pre-index, and post-index
/// 1. Immediate: Rt #offset           => Rt = Offset
/// 2. Standard:  Rt, [Rn {, #offset}] => Rt = m[Rn + Offset]
/// 3. Pre-index: Rt, [Rn, #offset]!   => Rt = mem[Rn + Offset]; Rt += offset
/// 4. Post-index: Rt, [Rn], #offset   => Rt = mem[Rn]; Rn += offset
fn parse_ldr(cs: &Capstone, instr: &Aarch64Insn) -> Vec<Stmt<Aarch64Regs>> {
    let mut stmts = Vec::new();
    let operands = get_aarch64_operands(cs, instr);
    let detail = cs.insn_detail(instr).expect("Unable to get detail");
    let writeback = detail.arch_detail().arm64().unwrap().writeback();
    // let writeback = arch_detail.writeback();
    match operands.len() {
        2 => {
            // Cases 1,2,3
            let dst = convert_operand_no_mem(cs, &operands[0]);
            let sz = dst.get_size();
            let src = convert_operand_any(cs, &operands[1], &sz);
            let stmt = Stmt::Unop(Unopcode::Mov, dst, src);
            stmts.push(stmt);
            // Pre-index writeback
            if writeback {
                let memargs = convert_op_to_mem(&operands[1]);
                stmts.push(writeback_pre_index(&memargs));
            }
        }
        3 => {
            // Case 4
            let dst = convert_operand_no_mem(cs, &operands[0]);
            let sz = dst.get_size();
            let src = convert_operand_any(cs, &operands[1], &sz);
            assert!(src.is_mem());
            let writeback_v = convert_operand_no_mem(cs, &operands[2]);
            let stmt = Stmt::Unop(Unopcode::Mov, dst, src);
            stmts.push(stmt);
            if writeback {
                let memargs = convert_op_to_mem(&operands[1]);
                stmts.push(writeback_post_index(&memargs, &writeback_v))
            }
        }
        _ => panic!("ldr, weird number of arguments"),
    }
    stmts
}

fn parse_str(cs: &Capstone, instr: &Aarch64Insn) -> Vec<Stmt<Aarch64Regs>> {
    let mut stmts = Vec::new();
    let operands = get_aarch64_operands(cs, instr);
    let detail = cs.insn_detail(instr).expect("Unable to get detail");
    let writeback = detail.arch_detail().arm64().unwrap().writeback();

    match operands.len() {
        2 => {
            let src = convert_operand_no_mem(cs, &operands[0]);
            let sz = src.get_size();
            let dst = convert_operand_any(cs, &operands[1], &sz);
            let stmt = Stmt::Unop(Unopcode::Mov, dst, src);
            stmts.push(stmt);
            if writeback {
                let memargs = convert_op_to_mem(&operands[1]);
                stmts.push(writeback_pre_index(&memargs));
            }
        }
        3 => {
            //TODO: this is post-index. Need to update rsp
            let src = convert_operand_no_mem(cs, &operands[0]);
            let sz = src.get_size();
            let dst = convert_operand_any(cs, &operands[1], &sz);
            assert!(dst.is_mem());
            let writeback_v = convert_operand_no_mem(cs, &operands[2]);
            let stmt = Stmt::Unop(Unopcode::Mov, dst, src);
            stmts.push(stmt);
            if writeback {
                let memargs = convert_op_to_mem(&operands[1]);
                stmts.push(writeback_post_index(&memargs, &writeback_v))
            }
        }
        _ => panic!("str, weird number of arguments"),
    }
    stmts
}

/// Parse the ldp (load pair) instruction
/// This instruction has two forms: a pre-index and a post-index form
/// Pre-index: LDP <Wt1>, <Wt2>, [<Xn|SP>, #<imm>]!
/// Post-index: LDP <Xt1>, <Xt2>, [<Xn|SP>], #<imm>
fn parse_ldp(cs: &Capstone, instr: &Aarch64Insn) -> Vec<Stmt<Aarch64Regs>> {
    let mut instrs = Vec::new();
    let operands = get_aarch64_operands(cs, instr);
    assert_eq!(operands.len(), 4);
    let dst0 = convert_operand_no_mem(cs, &operands[0]);
    let dst1 = convert_operand_no_mem(cs, &operands[1]);
    let sz = dst0.get_size();
    let src = convert_operand_mem(cs, &operands[2], &sz);
    let offset = convert_operand_no_mem(cs, &operands[3]);
    let off = offset.to_imm();
    let i0 = Stmt::Unop(Unopcode::Mov, dst0, src.add_imm(off));
    let i1 = Stmt::Binop(
        Binopcode::Add,
        Value::Reg(X31, Size64),
        Value::Reg(X31, Size64),
        Value::from(8),
    );
    let i2 = Stmt::Unop(Unopcode::Mov, dst1, src.add_imm(off));
    let i3 = Stmt::Binop(
        Binopcode::Add,
        Value::Reg(X31, Size64),
        Value::Reg(X31, Size64),
        Value::from(8),
    );
    instrs.push(i0);
    instrs.push(i1);
    instrs.push(i2);
    instrs.push(i3);
    return instrs;
}

fn parse_stp(cs: &Capstone, instr: &Aarch64Insn) -> Vec<Stmt<Aarch64Regs>> {
    let mut instrs = Vec::new();
    let operands = get_aarch64_operands(cs, instr);
    assert_eq!(operands.len(), 3);
    let src0 = convert_operand_no_mem(cs, &operands[0]);
    let src1 = convert_operand_no_mem(cs, &operands[1]);
    let sz = src0.get_size();
    let dst = convert_operand_mem(cs, &operands[2], &sz);
    // let offset = convert_operand_no_mem(cs, &operands[3]);
    // let off = offset.to_imm();

    let i0 = Stmt::Unop(Unopcode::Mov, src0, dst.clone());
    let i1 = Stmt::Binop(
        Binopcode::Add,
        Value::Reg(X31, Size64),
        Value::Reg(X31, Size64),
        Value::from(-8),
    );
    let i2 = Stmt::Unop(Unopcode::Mov, src1, dst);
    let i3 = Stmt::Binop(
        Binopcode::Add,
        Value::Reg(X31, Size64),
        Value::Reg(X31, Size64),
        Value::from(-8),
    );
    instrs.push(i0);
    instrs.push(i1);
    instrs.push(i2);
    instrs.push(i3);
    return instrs;
}

fn parse_cmp(cs: &Capstone, instr: &Aarch64Insn) -> Stmt<Aarch64Regs> {
    let operands = get_aarch64_operands(cs, instr);
    assert_eq!(operands.len(), 2);
    let src0 = convert_operand_no_mem(cs, &operands[0]);
    let src1 = convert_operand_no_mem(cs, &operands[1]);
    return Stmt::Binop(Binopcode::Cmp, Value::Reg(Nzcv, Size8), src0, src1);
}

fn parse_sub(cs: &Capstone, instr: &Aarch64Insn) -> Stmt<Aarch64Regs> {
    parse_arith_binop(cs, instr, Binopcode::Sub)
}

fn parse_add(cs: &Capstone, instr: &Aarch64Insn) -> Stmt<Aarch64Regs> {
    parse_arith_binop(cs, instr, Binopcode::Add)
}

fn parse_arith_binop(cs: &Capstone, instr: &Aarch64Insn, opcode: Binopcode) -> Stmt<Aarch64Regs> {
    let operands = get_aarch64_operands(cs, instr);
    assert_eq!(operands.len(), 3);
    let dst = convert_operand_no_mem(cs, &operands[0]);
    let src0 = convert_operand_no_mem(cs, &operands[1]);
    let src1 = convert_operand_no_mem(cs, &operands[2]);
    return Stmt::Binop(opcode, dst, src0, src1);
}

pub fn lift(
    cs: &Capstone,
    instr: &Aarch64Insn,
    addr: &u64,
    metadata: &VwMetadata,
    strict: bool,
) -> Vec<Stmt<Aarch64Regs>> {
    let mut instrs = Vec::new();
    match instr.mnemonic().unwrap() {
        "ret" => instrs.push(Stmt::Ret),
        "mov" => instrs.push(unop(cs, Unopcode::Mov, instr)),
        "blr" => instrs.push(parse_call(cs, instr)),
        "ldur" => instrs.push(parse_ldur(cs, instr)),
        "stur" => instrs.push(parse_stur(cs, instr)),
        "ldr" => instrs.extend(parse_ldr(cs, instr)), // ldr vs ldur?
        "str" => instrs.extend(parse_str(cs, instr)), // str vs stur?
        "ldp" => instrs.extend(parse_ldp(cs, instr)),
        "stp" => instrs.extend(parse_stp(cs, instr)),
        "cmp" => instrs.push(parse_cmp(cs, instr)),
        "sub" => instrs.push(parse_sub(cs, instr)),
        "add" => instrs.push(parse_add(cs, instr)),
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
            panic!("{}", err);
        }
    }
}

fn ends_in_check(block: &VW_Block, instrs: &capstone::Instructions, buf: &[u8]) -> bool {
    let last_instr = instrs.last().unwrap();
    let bytes_read = last_instr.address() + last_instr.bytes().len() as u64;
    if last_instr.mnemonic() == Some("b.hs") {
        if bytes_read <= block.end {
            let rest = &buf[(block.start + bytes_read) as usize..(block.end as usize)];
            // println!("ends_in_check!");
            return &rest[0..4] == [0, 0, 160, 212];
        }
    }
    false
}

fn ends_in_b(
    cs: &Capstone,
    block: &VW_Block,
    instrs: &capstone::Instructions,
    buf: &[u8],
) -> Option<i64> {
    let last_instr = instrs.last().unwrap();
    let bytes_read = last_instr.address() + last_instr.bytes().len() as u64;
    // println!("Ends in b: {:x} {:?} {:?}", last_instr.address(), bytes_read, block.end);
    if last_instr.mnemonic() == Some("b") {
        if bytes_read <= block.end {
            let operands = get_aarch64_operands(cs, last_instr);
            assert_eq!(operands.len(), 1);
            if let Arm64OperandType::Imm(i) = &operands[0].op_type {
                return Some(*i);
                // println!("ends_in_b: {:x}", i);
            }
            // 1. extract branch target
            // 2. get rest starting at that branch target
            // 3. return true
            // let rest = &buf[(block.start + bytes_read) as usize..(block.end as usize)];
            // return true;
        }
    }
    None
}

fn lift_block(
    cs: &Capstone,
    module: &VwModule,
    block: &VW_Block,
    buf: &[u8],
    strict: bool,
) -> IRBlock<Aarch64Regs> {
    // block addrs should not be used to index into buf since buf is just the function's code
    // and block addrs are the global address
    // block range is inclusive, range here is excl
    let mut instrs = disas_aarch64(&cs, buf, block.start);
    if instrs.len() == 0 {
        return Vec::new();
    }

    let mut block_ir = parse_instrs(&cs, &instrs, &module.metadata, strict);

    // Check until fixed point
    while ends_in_b(&cs, &block, &instrs, buf).is_some() || ends_in_check(&block, &instrs, buf) {
        if let Some(imm) = ends_in_b(&cs, &block, &instrs, buf) {
            instrs = disas_aarch64(
                &cs,
                &buf[(imm as u64 - block.start) as usize..],
                imm as u64, // add block.start?
            );

            let new_ir = parse_instrs(&cs, &instrs, &module.metadata, strict);
            block_ir.extend(new_ir);
        }

        // Deal with undefined instructions stopping disassembly
        if ends_in_check(&block, &instrs, buf) {
            let last_instr = instrs.last().unwrap();
            // println!("ends_in_check! {:x}", last_instr.address());
            let bytes_read = last_instr.address() + last_instr.bytes().len() as u64;
            instrs = disas_aarch64(
                &cs,
                &buf[(bytes_read + 4) as usize..],
                block.start + bytes_read + 4,
            );
            let new_ir = parse_instrs(&cs, &instrs, &module.metadata, strict);
            block_ir.extend(new_ir);
        }
    }
    block_ir
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

    // for idx in (0..text_segment.data.len()).step_by(4) {
    //     let mut v = &text_segment.data[idx..(idx + 4)];
    //     let num = v.read_u32::<BigEndian>().unwrap();
    //     println!("instr: 0x{:x}: {:x}", idx, num);
    // }

    for block_addr in g.nodes() {
        let block = cfg.get_block(block_addr);
        let buf_start = block.start as usize;
        let buf_end = block.end as usize;
        // block range is inclusive, range here is excl
        let buf = &text_segment.data[buf_start..=buf_end];
        // println!("block addr [{:x}:{:x}]", block.start, block.end);
        let block_ir = lift_block(&cs, module, &block, buf, strict);

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

    /// Lucet (which has a pinned heap register) does not support aarch64
    fn pinned_heap_reg() -> Self {
        unimplemented!()
    }

    fn pinned_vmctx_reg() -> Self {
        X0
    }

    fn is_caller_saved(&self) -> bool {
        match self {
            X0 | X1 | X2 | X3 | X4 | X5 | X6 | X7 | X8 | X9 | X10 | X11 | X12 | X13 | X14 | X15
            | X16 | X17 | X18 | X30 => true,
            _ => false,
        }
    }
}
