use yaxpeax_x86::long_mode::Opcode::*;
use std::collections::HashMap;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use yaxpeax_core::memory::repr::process::ModuleData;
use yaxpeax_x86::long_mode::{Arch as AMD64, Operand, RegSpec, RegisterBank, Opcode};
use yaxpeax_arch::Arch;
use yaxpeax_core::arch::InstructionSpan;

pub enum ImmType {
    Signed,
    Unsigned
}

pub enum ValSize {
    Size8,
    Size16,
    Size32,
    Size64
}

impl ValSize{
    pub fn to_u32(&self) -> u32{
        match self{
        Size8 => 8,
        Size16 => 16,
        Size32 => 32,
        Size64 => 64
        }
    }
}

pub fn valsize(num : u32) -> ValSize{
    match num{
        8 => ValSize::Size8,
        16 => ValSize::Size16,
        32 => ValSize::Size32,
        64 => ValSize::Size64,
        _ => unimplemented!("")
    }
}

pub fn mk_value_i64(num : i64) -> Value{
    Value::Imm(ImmType::Signed, ValSize::Size64, num)
}

pub enum MemArgs {
    Mem1Arg(MemArg),
    Mem2Args(MemArg, MemArg),
    Mem3Args(MemArg, MemArg, MemArg)
}

pub enum MemArg {
    Reg(u8, ValSize),
    Imm(ImmType, ValSize, i64) //signed, size, const
}

pub enum Value {
    Mem(ValSize, MemArgs),
    Reg(u8, ValSize),
    Imm(ImmType, ValSize, i64) //signed, size, const
}

// pub enum DstValue {
//     Mem(MemArgs),
//     Reg(u8, ValSize),
// }

pub enum Stmt {
    Clear(Value),
    Unop(Unopcode, Value, Value),
    Binop(Binopcode, Value, Value, Value),
    Undefined,
    Ret,
    Branch(yaxpeax_x86::long_mode::Opcode, Value),
    Call(Value)
}

impl Stmt{
    pub fn width(&self) -> u32{
        unimplemented!("Width not implemented")
    }
}

pub enum Unopcode {
    Mov,
}

pub enum Binopcode {
    Test,
    Rol,
    Cmp,
    Shl,
    And,
    Add,
    Sub,
}

fn convert_reg(reg : yaxpeax_x86::long_mode::RegSpec) -> Value{
    let size = match reg.bank{
        RegisterBank::Q => ValSize::Size64,
        RegisterBank::D => ValSize::Size32,
        RegisterBank::W => ValSize::Size16,
        RegisterBank::B => ValSize::Size8,
        RegisterBank::rB => ValSize::Size8,
        _ => panic!("Unknown register bank: {:?}", reg.bank)
    };
    Value::Reg(reg.num, size)
}

fn convert_memarg_reg(reg : yaxpeax_x86::long_mode::RegSpec) -> MemArg{
    let size = match reg.bank{
        RegisterBank::Q => ValSize::Size64,
        RegisterBank::D => ValSize::Size32,
        RegisterBank::W => ValSize::Size16,
        RegisterBank::B => ValSize::Size8,
        _ => panic!("Unknown register bank: {:?}", reg.bank)
    };
    MemArg::Reg(reg.num, size)
}


fn convert_operand(op : yaxpeax_x86::long_mode::Operand) -> Value{
    match op{
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
        Operand::DisplacementU32(imm) => Value::Mem(ValSize::Size64, MemArgs::Mem1Arg(MemArg::Imm(ImmType::Unsigned, ValSize::Size32, imm as i64))), //mem[c]
        Operand::DisplacementU64(imm) => Value::Mem(ValSize::Size64, MemArgs::Mem1Arg(MemArg::Imm(ImmType::Unsigned, ValSize::Size64, imm as i64))), //mem[c]
        Operand::RegDeref(reg) => Value::Mem(valsize(reg.width() as u32), MemArgs::Mem1Arg(convert_memarg_reg(reg) )), // mem[reg]
        Operand::RegDisp(reg, imm) => Value::Mem(valsize(reg.width() as u32), MemArgs::Mem2Args(convert_memarg_reg(reg), MemArg::Imm(ImmType::Signed, ValSize::Size32, imm as i64)) ), //mem[reg + c]
        Operand::RegIndexBase(reg1, reg2) => Value::Mem(valsize(reg1.width() as u32), MemArgs::Mem2Args(convert_memarg_reg(reg1), convert_memarg_reg(reg2)) ), // mem[reg1 + reg2]
        Operand::RegIndexBaseDisp(reg1, reg2, imm) => Value::Mem(valsize(reg1.width() as u32), MemArgs::Mem3Args(convert_memarg_reg(reg1), convert_memarg_reg(reg2), MemArg::Imm(ImmType::Signed, ValSize::Size32, imm as i64)) ), //mem[reg1 + reg2 + c]
        Operand::RegScale(_,_) => panic!("Memory operations with scaling prohibited"), // mem[reg * c]
        Operand::RegScaleDisp(_,_,_) => panic!("Memory operations with scaling prohibited"), //mem[reg*c1 + c2]
        Operand::RegIndexBaseScale(_,_,_) => panic!("Memory operations with scaling prohibited"),//mem[reg1 + reg2*c]
        Operand::RegIndexBaseScaleDisp(_,_,_,_) => panic!("Memory operations with scaling prohibited"),//mem[reg1 + reg2*c1 + c2]
        Operand::Nothing => panic!("Nothing Operand?"),
    }
}


fn clear_dst(instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    Stmt::Clear(convert_operand(instr.operand(0)))
}

fn unop(opcode: Unopcode, instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    Stmt::Unop(opcode, convert_operand(instr.operand(0)), convert_operand(instr.operand(1)))
}

fn binop(opcode: Binopcode, instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    Stmt::Binop(opcode, convert_operand(instr.operand(0)), convert_operand(instr.operand(1)), convert_operand(instr.operand(1)))
}

fn branch(instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    Stmt::Branch(instr.opcode, convert_operand(instr.operand(0)))
}

fn call(instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    Stmt::Call(convert_operand(instr.operand(0)))
}

fn lea(instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    let dst = instr.operand(0);
    match convert_operand(instr.operand(1)){
        Value::Mem(_, memargs) => 
            match memargs {
                MemArgs::Mem1Arg(arg) => match arg{
                    MemArg::Imm(_,_,val) => unop(Unopcode::Mov, instr),
                    _ => clear_dst(instr)
                },
                _ => clear_dst(instr)
            }, 
        _ => panic!("Illegal lea")
    }
}


pub fn lift(instr : &yaxpeax_x86::long_mode::Instruction) -> Vec<Stmt>{
    let mut instrs = Vec::new();
    println!("{:?}", instr);
    match instr.opcode{
        Opcode::MOV => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVSX => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVSXD => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVSD => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVD => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::MOVQ => instrs.push(unop(Unopcode::Mov, instr)),
        Opcode::LEA => instrs.push(lea(instr)),

        Opcode::TEST => instrs.push(binop(Binopcode::Test,instr)), 
        Opcode::CMP => instrs.push(binop(Binopcode::Cmp,instr)),
        Opcode::AND => instrs.push(binop(Binopcode::And,instr)), 
        Opcode::ADD => instrs.push(binop(Binopcode::Add,instr)), 
        Opcode::SUB => instrs.push(binop(Binopcode::Sub,instr)),
        Opcode::SHL => instrs.push(binop(Binopcode::Shl,instr)),

        Opcode::UD2 => instrs.push(Stmt::Undefined),
        
        Opcode::RETURN => instrs.push(Stmt::Ret), 

        Opcode::JMP => instrs.push(branch(instr)),
        Opcode::JO|Opcode::JNO|Opcode::JB|Opcode::JNB|Opcode::JZ|Opcode::JNZ|Opcode::JA|Opcode::JNA|Opcode::JS|Opcode::JNS|Opcode::JP|Opcode::JNP|Opcode::JL|Opcode::JGE|Opcode::JLE|Opcode::JG => instrs.push(branch(instr)),

        Opcode::CALL => instrs.push(call(instr)), 

        Opcode::PUSH => { let width = 64; //TODO: do not fix width
            instrs.push(Stmt::Binop(Binopcode::Sub, Value::Reg(4, ValSize::Size64), Value::Reg(4, ValSize::Size64), mk_value_i64(width / 8)));
            instrs.push(Stmt::Unop(Unopcode::Mov, Value::Mem(valsize(width as u32), MemArgs::Mem1Arg(MemArg::Reg(4, ValSize::Size64))), convert_operand(instr.operand(0))))
        },
        Opcode::POP => { let width = 64; //TODO: do not fix width
            instrs.push(Stmt::Unop(Unopcode::Mov, convert_operand(instr.operand(0)), Value::Mem(valsize(width as u32), MemArgs::Mem1Arg(MemArg::Reg(4, ValSize::Size64)))  ));
            instrs.push(Stmt::Binop(Binopcode::Add, Value::Reg(4, ValSize::Size64), Value::Reg(4, ValSize::Size64), mk_value_i64(width / 8)))
        },

        Opcode::OR => instrs.push(clear_dst(instr)),
        Opcode::SHR => instrs.push(clear_dst(instr)),
        Opcode::RCL => instrs.push(clear_dst(instr)),
        Opcode::RCR => instrs.push(clear_dst(instr)),
        Opcode::ROL => instrs.push(clear_dst(instr)), 
        Opcode::ROR => instrs.push(clear_dst(instr)),
        Opcode::CMOVA|Opcode::CMOVB|Opcode::CMOVG|Opcode::CMOVGE|Opcode::CMOVL|Opcode::CMOVLE|Opcode::CMOVNA|Opcode::CMOVNB|
        Opcode::CMOVNO|Opcode::CMOVNP|Opcode::CMOVNS|Opcode::CMOVNZ|Opcode::CMOVO|Opcode::CMOVP|Opcode::CMOVS|Opcode::CMOVZ => instrs.push(clear_dst(instr)), 
        SETO|SETNO|SETB|SETAE|SETZ|SETNZ|SETBE|SETA|SETS|
        SETNS|SETP|SETNP|SETL|SETGE|SETLE|SETG => instrs.push(clear_dst(instr)),
        Opcode::XOR => instrs.push(clear_dst(instr)),
        Opcode::SAR => instrs.push(clear_dst(instr)),
        Opcode::ADC => instrs.push(clear_dst(instr)), 
        Opcode::XOR => instrs.push(clear_dst(instr)),
        Opcode::ROUNDSS => instrs.push(clear_dst(instr)),
        Opcode::MUL => instrs.push(clear_dst(instr)),
        Opcode::MOVSS => instrs.push(clear_dst(instr)),
        Opcode::IMUL => instrs.push(clear_dst(instr)),  
        Opcode::DIV => instrs.push(clear_dst(instr)), 
        Opcode::XORPD => instrs.push(clear_dst(instr)),
        Opcode::MUL => instrs.push(clear_dst(instr)),
        Opcode::POR => instrs.push(clear_dst(instr)),
        Opcode::PSHUFB => instrs.push(clear_dst(instr)),
        Opcode::PSHUFD => instrs.push(clear_dst(instr)),
        Opcode::PTEST => instrs.push(clear_dst(instr)),
        Opcode::PXOR => instrs.push(clear_dst(instr)),
        Opcode::ANDNPS => instrs.push(clear_dst(instr)),  
        Opcode::XORPS => instrs.push(clear_dst(instr)), 
        Opcode::XORPD => instrs.push(clear_dst(instr)),
        Opcode::CMPPD => instrs.push(clear_dst(instr)),
        Opcode::CMPPS => instrs.push(clear_dst(instr)),
        Opcode::ANDPS => instrs.push(clear_dst(instr)),
        Opcode::ORPS => instrs.push(clear_dst(instr)),
        Opcode::MOVAPS => instrs.push(clear_dst(instr)),
        Opcode::DIVSD => instrs.push(clear_dst(instr)),
        Opcode::MULSS => instrs.push(clear_dst(instr)),
        Opcode::ADDSD => instrs.push(clear_dst(instr)),
        Opcode::UCOMISD => instrs.push(clear_dst(instr)),
        Opcode::SUBSS => instrs.push(clear_dst(instr)),
        Opcode::ROUNDSD => instrs.push(clear_dst(instr)),
        Opcode::NOT => instrs.push(clear_dst(instr)),
        Opcode::UCOMISS => instrs.push(clear_dst(instr)),
        Opcode::POPCNT => instrs.push(clear_dst(instr)),
        Opcode::SUBSD => instrs.push(clear_dst(instr)),
        Opcode::MULSD => instrs.push(clear_dst(instr)),
        Opcode::DIVSS => instrs.push(clear_dst(instr)),
        Opcode::IDIV => instrs.push(clear_dst(instr)),
        Opcode::ANDNPS => instrs.push(clear_dst(instr)),
        Opcode::LZCNT => instrs.push(clear_dst(instr)),
        Opcode::ANDPS => instrs.push(clear_dst(instr)),
        Opcode::DIV => instrs.push(clear_dst(instr)),
        Opcode::DIVPD => instrs.push(clear_dst(instr)),
        Opcode::DIVPS => instrs.push(clear_dst(instr)),
        Opcode::DIVSD => instrs.push(clear_dst(instr)),
        Opcode::DIVSS => instrs.push(clear_dst(instr)),
        Opcode::IDIV => instrs.push(clear_dst(instr)),
        Opcode::IMUL => instrs.push(clear_dst(instr)),
        Opcode::XORPD => instrs.push(clear_dst(instr)),
        Opcode::XORPS => instrs.push(clear_dst(instr)),
        Opcode::BLENDVPS => instrs.push(clear_dst(instr)),
        Opcode::BLENDVPD => instrs.push(clear_dst(instr)),
        Opcode::MAXPD => instrs.push(clear_dst(instr)),
        Opcode::MAXPS => instrs.push(clear_dst(instr)),
        Opcode::MAXSD => instrs.push(clear_dst(instr)),
        Opcode::MAXSS => instrs.push(clear_dst(instr)),
        Opcode::MINPD => instrs.push(clear_dst(instr)),
        Opcode::MINPS => instrs.push(clear_dst(instr)),
        Opcode::MINSD => instrs.push(clear_dst(instr)),
        Opcode::MINSS => instrs.push(clear_dst(instr)),
        Opcode::MULPD => instrs.push(clear_dst(instr)),
        Opcode::MULPS => instrs.push(clear_dst(instr)),
        Opcode::MULSD => instrs.push(clear_dst(instr)),
        Opcode::MULSS => instrs.push(clear_dst(instr)),
        Opcode::ORPS => instrs.push(clear_dst(instr)),
        Opcode::PMULLW => instrs.push(clear_dst(instr)),
        Opcode::PMULLD => instrs.push(clear_dst(instr)),
        Opcode::CVTDQ2PS => instrs.push(clear_dst(instr)),
        Opcode::CVTSD2SS => instrs.push(clear_dst(instr)),
        Opcode::CVTSI2SD => instrs.push(clear_dst(instr)),
        Opcode::CVTSI2SS => instrs.push(clear_dst(instr)),
        Opcode::CVTSS2SD => instrs.push(clear_dst(instr)),
        Opcode::CVTTSD2SI => instrs.push(clear_dst(instr)),
        Opcode::CVTTSS2SI => instrs.push(clear_dst(instr)),
        Opcode::ADDPS | Opcode::ADDPD | Opcode::ADDSD | Opcode::ADDSS => instrs.push(clear_dst(instr)), 
        Opcode::PSLLW => instrs.push(clear_dst(instr)),
        Opcode::PSLLD => instrs.push(clear_dst(instr)),
        Opcode::PSLLQ => instrs.push(clear_dst(instr)),
        Opcode::PSRLW => instrs.push(clear_dst(instr)),
        Opcode::PSRLD => instrs.push(clear_dst(instr)),
        Opcode::PSRLQ => instrs.push(clear_dst(instr)),
        Opcode::PSRAW => instrs.push(clear_dst(instr)),
        Opcode::PSRAD => instrs.push(clear_dst(instr)),
        Opcode::PSUBB => instrs.push(clear_dst(instr)),
        Opcode::PSUBW => instrs.push(clear_dst(instr)),
        Opcode::PSUBD => instrs.push(clear_dst(instr)),
        Opcode::PSUBQ => instrs.push(clear_dst(instr)),
        Opcode::PSUBSB => instrs.push(clear_dst(instr)),
        Opcode::PSUBSW => instrs.push(clear_dst(instr)),
        Opcode::PSUBUSB => instrs.push(clear_dst(instr)),
        Opcode::PSUBUSW => instrs.push(clear_dst(instr)),
        Opcode::PUNPCKHBW => instrs.push(clear_dst(instr)),
        Opcode::PUNPCKHWD => instrs.push(clear_dst(instr)),
        Opcode::PUNPCKHDQ => instrs.push(clear_dst(instr)),
        Opcode::PUNPCKHQDQ => instrs.push(clear_dst(instr)),
        Opcode::PUNPCKLBW => instrs.push(clear_dst(instr)),
        Opcode::PUNPCKLWD => instrs.push(clear_dst(instr)),
        Opcode::PUNPCKLDQ => instrs.push(clear_dst(instr)),
        Opcode::PUNPCKLQDQ => instrs.push(clear_dst(instr)),
        Opcode::PACKSSWB => instrs.push(clear_dst(instr)),
        Opcode::PACKSSDW => instrs.push(clear_dst(instr)),
        Opcode::PADDB => instrs.push(clear_dst(instr)),
        Opcode::PADDD => instrs.push(clear_dst(instr)),
        Opcode::PADDQ => instrs.push(clear_dst(instr)),
        Opcode::PADDW => instrs.push(clear_dst(instr)),
        Opcode::PADDSB => instrs.push(clear_dst(instr)),
        Opcode::PADDSW => instrs.push(clear_dst(instr)),
        Opcode::PADDUSB => instrs.push(clear_dst(instr)),
        Opcode::PADDUSW => instrs.push(clear_dst(instr)),
        Opcode::PAND => instrs.push(clear_dst(instr)),    
        Opcode::PANDN => instrs.push(clear_dst(instr)),
        Opcode::PAVGB => instrs.push(clear_dst(instr)),
        Opcode::PAVGW => instrs.push(clear_dst(instr)),
        Opcode::PCMPEQB => instrs.push(clear_dst(instr)),
        Opcode::PCMPEQD => instrs.push(clear_dst(instr)),
        Opcode::PCMPEQQ => instrs.push(clear_dst(instr)),
        Opcode::PCMPEQW => instrs.push(clear_dst(instr)),
        Opcode::PCMPGTB => instrs.push(clear_dst(instr)),
        Opcode::PCMPGTD => instrs.push(clear_dst(instr)),
        Opcode::PCMPGTQ => instrs.push(clear_dst(instr)),
        Opcode::PCMPGTW => instrs.push(clear_dst(instr)),
        Opcode::PEXTRB => instrs.push(clear_dst(instr)),
        Opcode::PEXTRW => instrs.push(clear_dst(instr)),
        Opcode::PINSRB => instrs.push(clear_dst(instr)),
        Opcode::PINSRW => instrs.push(clear_dst(instr)),
        Opcode::PMAXSB => instrs.push(clear_dst(instr)),
        Opcode::PMAXSW => instrs.push(clear_dst(instr)),
        Opcode::PMAXUB => instrs.push(clear_dst(instr)),
        Opcode::PMAXUD => instrs.push(clear_dst(instr)),
        Opcode::PMAXUW => instrs.push(clear_dst(instr)),
        Opcode::PMINSB => instrs.push(clear_dst(instr)),
        Opcode::PMINSD => instrs.push(clear_dst(instr)),
        Opcode::PMINSW => instrs.push(clear_dst(instr)),
        Opcode::PMINUB => instrs.push(clear_dst(instr)),
        Opcode::PMINUD => instrs.push(clear_dst(instr)),
        Opcode::PMINUW => instrs.push(clear_dst(instr)),
        Opcode::PMOVSXBW => instrs.push(clear_dst(instr)),
        Opcode::PMOVSXWD => instrs.push(clear_dst(instr)),
        Opcode::PMOVSXDQ => instrs.push(clear_dst(instr)),
        Opcode::PMOVZXBW => instrs.push(clear_dst(instr)),
        Opcode::PMOVZXWD => instrs.push(clear_dst(instr)),
        Opcode::PMOVZXDQ => instrs.push(clear_dst(instr)),
        Opcode::SQRTPD => instrs.push(clear_dst(instr)),
        Opcode::SQRTPS => instrs.push(clear_dst(instr)),
        Opcode::SQRTSD => instrs.push(clear_dst(instr)),
        Opcode::SQRTSS => instrs.push(clear_dst(instr)),
        Opcode::MOVLPS => instrs.push(clear_dst(instr)),
        Opcode::MOVAPS => instrs.push(clear_dst(instr)),
        Opcode::MOVLHPS => instrs.push(clear_dst(instr)),
        Opcode::MOVUPS => instrs.push(clear_dst(instr)),
        Opcode::SUBPD => instrs.push(clear_dst(instr)),
        Opcode::SUBPS => instrs.push(clear_dst(instr)),
        Opcode::SUBSD => instrs.push(clear_dst(instr)),
        Opcode::SUBSS | Opcode::TZCNT => instrs.push(clear_dst(instr)),
        Opcode::SBB | Opcode::BSR | Opcode::BSF => instrs.push(clear_dst(instr)),
        _ => unimplemented!()
    };
    instrs
}

pub type IRBlock = Vec< (u64, Vec<Stmt>) >;
pub type IRMap =  HashMap<u64, IRBlock>;

pub fn lift_cfg(program : &ModuleData, cfg : &ControlFlowGraph<u64>) -> IRMap{
    let mut irmap = IRMap::new();
    let g = &cfg.graph;
    for block_addr in g.nodes(){
        let mut block_ir : Vec<(u64, Vec<Stmt>)> = Vec::new();
        let block = cfg.get_block(block_addr);
        let mut iter = program.instructions_spanning(<AMD64 as Arch>::Decoder::default(), block.start, block.end);
        while let Some((addr, instr)) = iter.next() {
            // println!("{:x?}", addr);
            let ir = (addr,lift(instr));
            block_ir.push(ir);
        }
        irmap.insert(block_addr, block_ir);
    };
    irmap
}

