use crate::utils::LucetMetadata;
use std::collections::HashMap;
use yaxpeax_arch::Arch;
use yaxpeax_core::analyses::control_flow::VW_CFG;
use yaxpeax_core::arch::x86_64::analyses::data_flow::Location;
use yaxpeax_core::arch::InstructionSpan;
use yaxpeax_core::data::{Direction, ValueLocations};
use yaxpeax_core::memory::repr::process::ModuleData;
use yaxpeax_x86::long_mode::Opcode::*;
use yaxpeax_x86::long_mode::{Arch as AMD64, Opcode, Operand, RegisterBank};

#[derive(Debug, Clone)]
pub enum ImmType {
    Signed,
    Unsigned,
}
#[derive(Debug, Clone, Copy)]
pub enum ValSize {
    Size8,
    Size16,
    Size32,
    Size64,
    SizeOther,
}

impl ValSize {
    pub fn to_u32(&self) -> u32 {
        match self {
            ValSize::Size8 => 8,
            ValSize::Size16 => 16,
            ValSize::Size32 => 32,
            ValSize::Size64 => 64,
            ValSize::SizeOther => 64, //panic!("unknown size? {:?}")
        }
    }
}

pub fn valsize(num: u32) -> ValSize {
    match num {
        8 => ValSize::Size8,
        16 => ValSize::Size16,
        32 => ValSize::Size32,
        64 => ValSize::Size64,
        _ => unimplemented!("{:?}", num),
    }
}

pub fn mk_value_i64(num: i64) -> Value {
    Value::Imm(ImmType::Signed, ValSize::Size64, num)
}

#[derive(Debug, Clone)]
pub enum MemArgs {
    Mem1Arg(MemArg),
    Mem2Args(MemArg, MemArg),
    Mem3Args(MemArg, MemArg, MemArg),
    MemScale(MemArg, MemArg, MemArg),
}
#[derive(Debug, Clone)]
pub enum MemArg {
    Reg(u8, ValSize),
    Imm(ImmType, ValSize, i64), //signed, size, const
}
#[derive(Debug, Clone)]
pub enum Value {
    Mem(ValSize, MemArgs),
    Reg(u8, ValSize),
    Imm(ImmType, ValSize, i64), //signed, size, const
}

// pub enum DstValue {
//     Mem(MemArgs),
//     Reg(u8, ValSize),
// }
#[derive(Debug, Clone)]
pub enum Stmt {
    Clear(Value, Vec<Value>),
    Unop(Unopcode, Value, Value),
    Binop(Binopcode, Value, Value, Value),
    Undefined,
    Ret,
    Branch(yaxpeax_x86::long_mode::Opcode, Value),
    Call(Value),
    ProbeStack(u64),
}

impl Stmt {
    pub fn width(&self) -> u32 {
        unimplemented!("Width not implemented")
    }
}
#[derive(Debug, Clone)]
pub enum Unopcode {
    Mov,
}
#[derive(Debug, Clone)]
pub enum Binopcode {
    Test,
    Rol,
    Cmp,
    Shl,
    And,
    Add,
    Sub,
}

fn get_reg_size(reg: yaxpeax_x86::long_mode::RegSpec) -> ValSize {
    let size = match reg.bank {
        RegisterBank::Q => ValSize::Size64,
        RegisterBank::D => ValSize::Size32,
        RegisterBank::W => ValSize::Size16,
        RegisterBank::B => ValSize::Size8,
        RegisterBank::rB => ValSize::Size8,
        RegisterBank::RIP => panic!("Write to RIP: {:?}", reg.bank),
        RegisterBank::EIP => panic!("Write to EIP: {:?}", reg.bank),
        _ => ValSize::SizeOther, //xmm and ymm
    };
    return size;
}

fn convert_reg(reg: yaxpeax_x86::long_mode::RegSpec) -> Value {
    let size = get_reg_size(reg);
    Value::Reg(reg.num, size)
}

fn convert_memarg_reg(reg: yaxpeax_x86::long_mode::RegSpec) -> MemArg {
    let size = match reg.bank {
        RegisterBank::Q => ValSize::Size64,
        RegisterBank::D => ValSize::Size32,
        RegisterBank::W => ValSize::Size16,
        RegisterBank::B => ValSize::Size8,
        _ => panic!("Unknown register bank: {:?}", reg.bank),
    };
    MemArg::Reg(reg.num, size)
}

fn convert_operand(op: yaxpeax_x86::long_mode::Operand, memsize: ValSize) -> Value {
    match op {
        Operand::ImmediateI8(imm) => Value::Imm(ImmType::Signed, ValSize::Size8, imm as i64),
        Operand::ImmediateU8(imm) => Value::Imm(ImmType::Unsigned, ValSize::Size8, imm as i64),
        Operand::ImmediateI16(imm) => Value::Imm(ImmType::Signed, ValSize::Size16, imm as i64),
        Operand::ImmediateU16(imm) => Value::Imm(ImmType::Unsigned, ValSize::Size16, imm as i64),
        Operand::ImmediateU32(imm) => Value::Imm(ImmType::Unsigned, ValSize::Size32, imm as i64),
        Operand::ImmediateI32(imm) => Value::Imm(ImmType::Signed, ValSize::Size32, imm as i64),
        Operand::ImmediateU64(imm) => Value::Imm(ImmType::Unsigned, ValSize::Size64, imm as i64),
        Operand::ImmediateI64(imm) => Value::Imm(ImmType::Signed, ValSize::Size64, imm as i64),
        Operand::Register(reg) => convert_reg(reg),
        //u32 and u64 are address sizes
        Operand::DisplacementU32(imm) => Value::Mem(
            memsize,
            MemArgs::Mem1Arg(MemArg::Imm(ImmType::Unsigned, ValSize::Size32, imm as i64)),
        ), //mem[c]
        Operand::DisplacementU64(imm) => Value::Mem(
            memsize,
            MemArgs::Mem1Arg(MemArg::Imm(ImmType::Unsigned, ValSize::Size64, imm as i64)),
        ), //mem[c]
        Operand::RegDeref(reg) => Value::Mem(memsize, MemArgs::Mem1Arg(convert_memarg_reg(reg))), // mem[reg]
        Operand::RegDisp(reg, imm) => Value::Mem(
            memsize,
            MemArgs::Mem2Args(
                convert_memarg_reg(reg),
                MemArg::Imm(ImmType::Signed, ValSize::Size32, imm as i64),
            ),
        ), //mem[reg + c]
        Operand::RegIndexBase(reg1, reg2) => Value::Mem(
            memsize,
            MemArgs::Mem2Args(convert_memarg_reg(reg1), convert_memarg_reg(reg2)),
        ), // mem[reg1 + reg2]
        Operand::RegIndexBaseDisp(reg1, reg2, imm) => Value::Mem(
            memsize,
            MemArgs::Mem3Args(
                convert_memarg_reg(reg1),
                convert_memarg_reg(reg2),
                MemArg::Imm(ImmType::Signed, ValSize::Size32, imm as i64),
            ),
        ), //mem[reg1 + reg2 + c]
        Operand::RegScale(_, _) => panic!("Memory operations with scaling prohibited"), // mem[reg * c]
        Operand::RegScaleDisp(_, _, _) => panic!("Memory operations with scaling prohibited"), //mem[reg*c1 + c2]
        Operand::RegIndexBaseScale(reg1, reg2, scale) =>
        //mem[reg1 + reg2*c]
        {
            if scale == 1 {
                Value::Mem(
                    memsize,
                    MemArgs::Mem2Args(convert_memarg_reg(reg1), convert_memarg_reg(reg2)),
                )
            } else {
                Value::Mem(
                    memsize,
                    MemArgs::MemScale(
                        convert_memarg_reg(reg1),
                        convert_memarg_reg(reg2),
                        MemArg::Imm(ImmType::Signed, ValSize::Size32, scale as i64),
                    ),
                )
            }
        }
        Operand::RegIndexBaseScaleDisp(reg1, reg2, scale, imm) => {
            assert_eq!(scale, 1);
            Value::Mem(
                memsize,
                MemArgs::Mem3Args(
                    convert_memarg_reg(reg1),
                    convert_memarg_reg(reg2),
                    MemArg::Imm(ImmType::Signed, ValSize::Size32, imm as i64),
                ),
            )
        } //mem[reg1 + reg2*c1 + c2]
        Operand::Nothing => panic!("Nothing Operand?"),
    }
}

fn get_sources(instr: &yaxpeax_x86::long_mode::Instruction) -> Vec<Value> {
    match instr.operand_count() {
        0 => vec![],
        1 => vec![convert_operand(instr.operand(0), ValSize::Size32)],
        2 => vec![
            convert_operand(instr.operand(0), ValSize::Size32),
            convert_operand(instr.operand(1), ValSize::Size32),
        ],
        3 => vec![
            convert_operand(instr.operand(0), ValSize::Size32),
            convert_operand(instr.operand(1), ValSize::Size32),
            convert_operand(instr.operand(2), ValSize::Size32),
        ],
        4 => vec![
            convert_operand(instr.operand(0), ValSize::Size32),
            convert_operand(instr.operand(1), ValSize::Size32),
            convert_operand(instr.operand(2), ValSize::Size32),
            convert_operand(instr.operand(3), ValSize::Size32),
        ],
        _ => panic!("Too many arguments?"),
    }
}

fn clear_dst(instr: &yaxpeax_x86::long_mode::Instruction) -> Stmt {
    let uses_vec = <AMD64 as ValueLocations>::decompose(instr);
    let dsts: Vec<&Location> = uses_vec
        .iter()
        .filter_map(|(loc, dir)| match (loc, dir) {
            (Some(loc), Direction::Write) => Some(loc),
            _ => None,
        })
        .collect();
    // println!("dsts: {:?}", dsts);
    let srcs: Vec<Value> = get_sources(instr);
    Stmt::Clear(convert_operand(instr.operand(0), ValSize::Size8), srcs)
}

fn get_operand_size(op: yaxpeax_x86::long_mode::Operand) -> Option<ValSize> {
    match op {
        Operand::ImmediateI8(_) | Operand::ImmediateU8(_) => Some(ValSize::Size8),
        Operand::ImmediateI16(_) | Operand::ImmediateU16(_) => Some(ValSize::Size16),
        Operand::ImmediateU32(_) | Operand::ImmediateI32(_) => Some(ValSize::Size32),
        Operand::ImmediateU64(_) | Operand::ImmediateI64(_) => Some(ValSize::Size64),
        Operand::Register(reg) => Some(get_reg_size(reg)),
        //u32 and u64 are address sizes
        Operand::DisplacementU32(_)
        | Operand::DisplacementU64(_)
        | Operand::RegDeref(_)
        | Operand::RegDisp(_, _)
        | Operand::RegIndexBase(_, _)
        | Operand::RegIndexBaseDisp(_, _, _)
        | Operand::RegScale(_, _)
        | Operand::RegScaleDisp(_, _, _)
        | Operand::RegIndexBaseScale(_, _, _)
        | Operand::RegIndexBaseScaleDisp(_, _, _, _)
        | Operand::Nothing => None,
    }
}

fn unop(opcode: Unopcode, instr: &yaxpeax_x86::long_mode::Instruction) -> Stmt {
    let memsize = match (
        get_operand_size(instr.operand(0)),
        get_operand_size(instr.operand(1)),
    ) {
        (None, None) => panic!("Two Memory Args?"),
        (Some(x), None) => x,
        (None, Some(x)) => x,
        (Some(x), Some(_y)) => x,
    };
    Stmt::Unop(
        opcode,
        convert_operand(instr.operand(0), memsize),
        convert_operand(instr.operand(1), memsize),
    )
}

fn binop(opcode: Binopcode, instr: &yaxpeax_x86::long_mode::Instruction) -> Stmt {
    let memsize = match (
        get_operand_size(instr.operand(0)),
        get_operand_size(instr.operand(1)),
    ) {
        (None, None) => panic!("Two Memory Args?"),
        (Some(x), None) => x,
        (None, Some(x)) => x,
        (Some(x), Some(_y)) => x,
    };
    // if two operands than dst is src1
    if instr.operand_count() == 2 {
        Stmt::Binop(
            opcode,
            convert_operand(instr.operand(0), memsize),
            convert_operand(instr.operand(0), memsize),
            convert_operand(instr.operand(1), memsize),
        )
    } else {
        Stmt::Binop(
            opcode,
            convert_operand(instr.operand(0), memsize),
            convert_operand(instr.operand(1), memsize),
            convert_operand(instr.operand(2), memsize),
        )
    }
}

fn branch(instr: &yaxpeax_x86::long_mode::Instruction) -> Stmt {
    Stmt::Branch(
        instr.opcode,
        convert_operand(instr.operand(0), ValSize::Size64),
    )
}

fn call(instr: &yaxpeax_x86::long_mode::Instruction, _metadata: &LucetMetadata) -> Stmt {
    let dst = convert_operand(instr.operand(0), ValSize::Size64);
    Stmt::Call(dst)
}

fn lea(instr: &yaxpeax_x86::long_mode::Instruction, addr: &u64) -> Stmt {
    let dst = instr.operand(0);
    let src1 = instr.operand(1);
    if let Operand::RegDisp(reg, _imm) = src1 {
        if reg.bank == RegisterBank::RIP {
            //addr + instruction length + displacement
            let target = (*addr as i64) + (instr.length as i64) + (instr.disp as i64);
            return Stmt::Unop(
                Unopcode::Mov,
                convert_operand(dst, ValSize::SizeOther),
                Value::Imm(ImmType::Signed, ValSize::Size64, target),
            );
        }
    }
    match convert_operand(src1, get_operand_size(dst).unwrap()) {
        Value::Mem(_, memargs) => match memargs {
            MemArgs::Mem1Arg(arg) => match arg {
                MemArg::Imm(_, _, _val) => unop(Unopcode::Mov, instr),
                _ => clear_dst(instr),
            },
            _ => clear_dst(instr),
        },
        _ => panic!("Illegal lea"),
    }
}

pub fn lift(
    instr: &yaxpeax_x86::long_mode::Instruction,
    addr: &u64,
    metadata: &LucetMetadata,
) -> Vec<Stmt> {
    let mut instrs = Vec::new();
    //println!("{:?} {:?} instr", addr, instr);

    match instr.opcode {
        Opcode::MOV => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVSX => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVSXD => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVSD => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVD => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVQ => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVZX_b => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVSX_b => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVZX_w => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVSX_w => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::LEA => instrs.push(lea(instr, addr)),

        Opcode::TEST => instrs.push(binop(Binopcode::Test, instr)),
        Opcode::CMP => instrs.push(binop(Binopcode::Cmp, instr)),
        // Opcode::CMP => instrs.push(cmp(instr)),
        //instrs.push(Stmt::Clear(Value::Reg(16, ValSize::Size8), vec![]))
        Opcode::AND => instrs.push(binop(Binopcode::And, instr)),
        Opcode::ADD => instrs.push(binop(Binopcode::Add, instr)),
        Opcode::SUB => instrs.push(binop(Binopcode::Sub, instr)),
        Opcode::SHL => instrs.push(binop(Binopcode::Shl, instr)),

        Opcode::UD2 => instrs.push(Stmt::Undefined),

        Opcode::RETURN => instrs.push(Stmt::Ret),

        Opcode::JMP => instrs.push(branch(instr)),
        Opcode::JO
        | Opcode::JNO
        | Opcode::JB
        | Opcode::JNB
        | Opcode::JZ
        | Opcode::JNZ
        | Opcode::JA
        | Opcode::JNA
        | Opcode::JS
        | Opcode::JNS
        | Opcode::JP
        | Opcode::JNP
        | Opcode::JL
        | Opcode::JGE
        | Opcode::JLE
        | Opcode::JG => instrs.push(branch(instr)),

        Opcode::CALL => instrs.push(call(instr, metadata)),

        Opcode::PUSH => {
            let width = instr.operand(0).width();
            assert_eq!(width, 8); //8 bytes
            instrs.push(Stmt::Binop(
                Binopcode::Sub,
                Value::Reg(4, ValSize::Size64),
                Value::Reg(4, ValSize::Size64),
                mk_value_i64(width.into()),
            ));
            instrs.push(Stmt::Unop(
                Unopcode::Mov,
                Value::Mem(
                    valsize((width * 8) as u32),
                    MemArgs::Mem1Arg(MemArg::Reg(4, ValSize::Size64)),
                ),
                convert_operand(instr.operand(0), ValSize::SizeOther),
            ))
        }
        Opcode::POP => {
            let width = instr.operand(0).width();
            assert_eq!(width, 8); //8 bytes
            instrs.push(Stmt::Unop(
                Unopcode::Mov,
                convert_operand(instr.operand(0), ValSize::SizeOther),
                Value::Mem(
                    valsize((width * 8) as u32),
                    MemArgs::Mem1Arg(MemArg::Reg(4, ValSize::Size64)),
                ),
            ));
            instrs.push(Stmt::Binop(
                Binopcode::Add,
                Value::Reg(4, ValSize::Size64),
                Value::Reg(4, ValSize::Size64),
                mk_value_i64(width.into()),
            ))
        }

        Opcode::NOP | Opcode::FILD | Opcode::STD | Opcode::CLD | Opcode::STI => (),
        Opcode::IDIV | Opcode::DIV => {
            // instrs.push(Stmt::Clear(Value::Reg(16, ValSize::Size8), vec![]));
            instrs.push(Stmt::Clear(Value::Reg(0, ValSize::Size64), vec![])); // clear RAX
            instrs.push(Stmt::Clear(Value::Reg(2, ValSize::Size64), vec![])); // clear RDX
        }

        Opcode::XOR => {
            //XOR reg, reg => mov reg, 0
            if instr.operand_count() == 2 && instr.operand(0) == instr.operand(1) {
                instrs.push(Stmt::Unop(
                    Unopcode::Mov,
                    convert_operand(instr.operand(0), ValSize::Size64),
                    Value::Imm(ImmType::Signed, ValSize::Size64, 0),
                ));
            } else {
                instrs.push(clear_dst(instr))
            }
        }

        Opcode::OR
        | Opcode::SHR
        | Opcode::RCL
        | Opcode::RCR
        | Opcode::ROL
        | Opcode::ROR
        | Opcode::CMOVA
        | Opcode::CMOVB
        | Opcode::CMOVG
        | Opcode::CMOVGE
        | Opcode::CMOVL
        | Opcode::CMOVLE
        | Opcode::CMOVNA
        | Opcode::CMOVNB
        | Opcode::CMOVNO
        | Opcode::CMOVNP
        | Opcode::CMOVNS
        | Opcode::CMOVNZ
        | Opcode::CMOVO
        | Opcode::CMOVP
        | Opcode::CMOVS
        | Opcode::CMOVZ
        | SETO
        | SETNO
        | SETB
        | SETAE
        | SETZ
        | SETNZ
        | SETBE
        | SETA
        | SETS
        | SETNS
        | SETP
        | SETNP
        | SETL
        | SETGE
        | SETLE
        | SETG
        | Opcode::SAR
        | Opcode::ADC
        | Opcode::ROUNDSS
        | Opcode::MUL
        | Opcode::MOVSS
        | Opcode::IMUL
        | Opcode::XORPD
        | Opcode::POR
        | Opcode::PSHUFB
        | Opcode::PSHUFD
        | Opcode::PTEST
        | Opcode::PXOR
        | Opcode::ANDNPS
        | Opcode::XORPS
        | Opcode::CMPPD
        | Opcode::CMPPS
        | Opcode::ANDPS
        | Opcode::ORPS
        | Opcode::MOVAPS
        | Opcode::DIVSD
        | Opcode::MULSS
        | Opcode::ADDSD
        | Opcode::UCOMISD
        | Opcode::SUBSS
        | Opcode::ROUNDSD
        | Opcode::NOT
        | Opcode::UCOMISS
        | Opcode::POPCNT
        | Opcode::SUBSD
        | Opcode::MULSD
        | Opcode::DIVSS
        | Opcode::LZCNT
        | Opcode::DIVPD
        | Opcode::DIVPS
        | Opcode::BLENDVPS
        | Opcode::BLENDVPD
        | Opcode::MAXPD
        | Opcode::MAXPS
        | Opcode::MAXSD
        | Opcode::MAXSS
        | Opcode::MINPD
        | Opcode::MINPS
        | Opcode::MINSD
        | Opcode::MINSS
        | Opcode::MULPD
        | Opcode::MULPS
        | Opcode::PMULLW
        | Opcode::PMULLD
        | Opcode::CVTDQ2PS
        | Opcode::CVTSD2SS
        | Opcode::CVTSI2SD
        | Opcode::CVTSI2SS
        | Opcode::CVTSS2SD
        | Opcode::CVTTSD2SI
        | Opcode::CVTTSS2SI
        | Opcode::ADDPS
        | Opcode::ADDPD
        | Opcode::ADDSS
        | Opcode::PSLLW
        | Opcode::PSLLD
        | Opcode::PSLLQ
        | Opcode::PSRLW
        | Opcode::PSRLD
        | Opcode::PSRLQ
        | Opcode::PSRAW
        | Opcode::PSRAD
        | Opcode::PSUBB
        | Opcode::PSUBW
        | Opcode::PSUBD
        | Opcode::PSUBQ
        | Opcode::PSUBSB
        | Opcode::PSUBSW
        | Opcode::PSUBUSB
        | Opcode::PSUBUSW
        | Opcode::PUNPCKHBW
        | Opcode::PUNPCKHWD
        | Opcode::PUNPCKHDQ
        | Opcode::PUNPCKHQDQ
        | Opcode::PUNPCKLBW
        | Opcode::PUNPCKLWD
        | Opcode::PUNPCKLDQ
        | Opcode::PUNPCKLQDQ
        | Opcode::PACKSSWB
        | Opcode::PACKSSDW
        | Opcode::PADDB
        | Opcode::PADDD
        | Opcode::PADDQ
        | Opcode::PADDW
        | Opcode::PADDSB
        | Opcode::PADDSW
        | Opcode::PADDUSB
        | Opcode::PADDUSW
        | Opcode::PAND
        | Opcode::PANDN
        | Opcode::PAVGB
        | Opcode::PAVGW
        | Opcode::PCMPEQB
        | Opcode::PCMPEQD
        | Opcode::PCMPEQQ
        | Opcode::PCMPEQW
        | Opcode::PCMPGTB
        | Opcode::PCMPGTD
        | Opcode::PCMPGTQ
        | Opcode::PCMPGTW
        | Opcode::PEXTRB
        | Opcode::PEXTRW
        | Opcode::PINSRB
        | Opcode::PINSRW
        | Opcode::PMAXSB
        | Opcode::PMAXSW
        | Opcode::PMAXUB
        | Opcode::PMAXUD
        | Opcode::PMAXUW
        | Opcode::PMINSB
        | Opcode::PMINSD
        | Opcode::PMINSW
        | Opcode::PMINUB
        | Opcode::PMINUD
        | Opcode::PMINUW
        | Opcode::PMOVSXBW
        | Opcode::PMOVSXWD
        | Opcode::PMOVSXDQ
        | Opcode::PMOVZXBW
        | Opcode::PMOVZXWD
        | Opcode::PMOVZXDQ
        | Opcode::SQRTPD
        | Opcode::SQRTPS
        | Opcode::SQRTSD
        | Opcode::SQRTSS
        | Opcode::MOVLPS
        | Opcode::MOVLHPS
        | Opcode::MOVUPS
        | Opcode::SUBPD
        | Opcode::SUBPS
        | Opcode::TZCNT
        | Opcode::SBB
        | Opcode::BSR
        | Opcode::BSF => instrs.push(clear_dst(instr)),
        _ => unimplemented!(),
    };
    instrs
}

pub type IRBlock = Vec<(u64, Vec<Stmt>)>;
pub type IRMap = HashMap<u64, IRBlock>;

fn is_probestack(
    instr: &yaxpeax_x86::long_mode::Instruction,
    addr: &u64,
    metadata: &LucetMetadata,
) -> bool {
    if let Opcode::CALL = instr.opcode {
        if let Value::Imm(_, _, offset) = convert_operand(instr.operand(0), ValSize::SizeOther) {
            // 5 = size of call instruction
            if 5 + offset + (*addr as i64) == metadata.lucet_probestack as i64 {
                return true;
            }
        }
    }
    false
}

fn extract_probestack_arg(instr: &yaxpeax_x86::long_mode::Instruction) -> Option<u64> {
    if let Opcode::MOV = instr.opcode {
        if let Value::Reg(0, ValSize::Size32) =
            convert_operand(instr.operand(0), ValSize::SizeOther)
        {
            if let Value::Imm(_, _, x) = convert_operand(instr.operand(1), ValSize::SizeOther) {
                if instr.operand_count() == 2 {
                    return Some(x as u64);
                }
            }
        }
    }
    None
}

fn check_probestack_suffix(instr: &yaxpeax_x86::long_mode::Instruction) -> bool {
    if let Opcode::SUB = instr.opcode {
        if let Value::Reg(4, ValSize::Size64) =
            convert_operand(instr.operand(0), ValSize::SizeOther)
        {
            //size is dummy
            if let Value::Reg(0, ValSize::Size64) =
                convert_operand(instr.operand(1), ValSize::SizeOther)
            {
                if instr.operand_count() == 2 {
                    return true;
                }
            }
        }
    }
    panic!("Broken Probestack?")
}

pub fn lift_cfg(program: &ModuleData, cfg: &VW_CFG, metadata: &LucetMetadata) -> IRMap {
    let mut irmap = IRMap::new();
    let g = &cfg.graph;
    for block_addr in g.nodes() {
        let mut block_ir: Vec<(u64, Vec<Stmt>)> = Vec::new();
        let block = cfg.get_block(block_addr);
        let mut iter = program.instructions_spanning(
            <AMD64 as Arch>::Decoder::default(),
            block.start,
            block.end,
        );
        let mut probestack_suffix = false;
        let mut x: Option<u64> = None;
        while let Some((addr, instr)) = iter.next() {
            if probestack_suffix {
                //1. fail if it isnt sub, rsp, rax
                //2. skip
                probestack_suffix = false;
                check_probestack_suffix(instr);

                continue;
            }
            if is_probestack(instr, &addr, &metadata) {
                match x {
                    Some(v) => {
                        // println!("Found Probestack: {:?} {:?} {:?}", v, v / 4096, ((v / 4096) + 1) * 4096);
                        let ir = (addr, vec![Stmt::ProbeStack(v)]);
                        block_ir.push(ir);
                        probestack_suffix = true;
                        continue;
                    }
                    None => panic!("probestack broken"),
                }
            }
            let ir = (addr, lift(instr, &addr, metadata));
            block_ir.push(ir);
            x = extract_probestack_arg(instr);
            // last_instr = Some(*instr);
        }
        irmap.insert(block_addr, block_ir);
    }
    irmap
}
