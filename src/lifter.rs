use yaxpeax_x86::long_mode::Opcode::*;
use std::collections::HashMap;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use yaxpeax_core::memory::repr::process::ModuleData;
use yaxpeax_x86::long_mode::{Arch as AMD64, Operand, RegSpec, RegisterBank};
use yaxpeax_arch::Arch;
use yaxpeax_core::arch::InstructionSpan;



//cfg = entrypoint, blocks, graph
//basic block = start, end
// instruction_span

//get_block


/*
pub struct Instruction {
    pub prefixes: Prefixes,
    modrm_rrr: RegSpec,
    modrm_mmm: RegSpec, // doubles as sib_base
    sib_index: RegSpec,
    vex_reg: RegSpec,
    scale: u8,
    length: u8,
    operand_count: u8,
    operands: [OperandSpec; 4],
    imm: u64,
    disp: u64,
    pub opcode: Opcode,
}


#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
enum OperandSpec {
    Nothing,
    // the register in modrm_rrr
    RegRRR,
    // the register in modrm_mmm (eg modrm mod bits were 11)
    RegMMM,
    // the register selected by vex-vvvv bits
    RegVex,
    // the register selected by a handful of avx2 vex-coded instructions,
    // stuffed in imm4.
    Reg4,
    ImmI8,
    ImmI16,
    ImmI32,
    ImmI64,
    ImmU8,
    ImmU16,
    ImmU32,
    ImmU64,
    // ENTER is a two-immediate instruction, where the first immediate is stored in the disp field.
    // for this case, a second immediate-style operand is needed.
    EnterFrameSize,
    DispU32,
    DispU64,
    Deref,
    Deref_rsi,
    Deref_rdi,
    RegDisp,
    RegScale,
    RegIndexBase,
    RegIndexBaseDisp,
    RegScaleDisp,
    RegIndexBaseScale,
    RegIndexBaseScaleDisp
}

pub struct RegSpec {
    pub num: u8, -- identifier
    pub bank: RegisterBank -- size
}

*/

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

pub enum MemArgs {
    Mem1Arg(MemArg),
    Mem2Args(MemArg, MemArg),
    Mem3Args(MemArg, MemArg, MemArg)
}

pub enum MemArg {
    Reg(u8, ValSize),
    Imm(ImmType, ValSize, u64) //signed, size, const
}

pub enum Value {
    Mem(MemArgs),
    Reg(u8, ValSize),
    Imm(ImmType, ValSize, u64) //signed, size, const
}

pub enum Stmt {
    Clear(yaxpeax_x86::long_mode::Opcode, Value),
    Unop(yaxpeax_x86::long_mode::Opcode, Value, Value),
    Binop(yaxpeax_x86::long_mode::Opcode, Value, Value, Value),
    Undefined,
    Ret,
    Branch(yaxpeax_x86::long_mode::Opcode, Value),
    Call(Value),
    Pop(Value),
    Push(Value)
}
fn convert_reg(reg : yaxpeax_x86::long_mode::RegSpec) -> Value{
    unimplemented!()
}

fn convert_memarg_reg(reg : yaxpeax_x86::long_mode::RegSpec) -> MemArg{
    unimplemented!()
}

fn convert_operand(op : yaxpeax_x86::long_mode::Operand) -> Value{
    match op{
        Operand::ImmediateI8(imm) => Value::Imm(ImmType::Signed, ValSize::Size8, imm as u64),
        Operand::ImmediateU8(imm) => Value::Imm(ImmType::Unsigned, ValSize::Size8, imm as u64),
        Operand::ImmediateI16(imm) => Value::Imm(ImmType::Signed, ValSize::Size16, imm as u64),
        Operand::ImmediateU16(imm) => Value::Imm(ImmType::Unsigned, ValSize::Size16, imm as u64),
        Operand::ImmediateU32(imm) => Value::Imm(ImmType::Unsigned, ValSize::Size32, imm as u64),
        Operand::ImmediateI32(imm) => Value::Imm(ImmType::Signed, ValSize::Size32, imm as u64),
        Operand::ImmediateU64(imm) => Value::Imm(ImmType::Unsigned, ValSize::Size64, imm as u64),
        Operand::ImmediateI64(imm) => Value::Imm(ImmType::Signed, ValSize::Size64, imm as u64),
        Operand::Register(reg) => convert_reg(reg),
        Operand::DisplacementU32(imm) => Value::Mem(MemArgs::Mem1Arg(MemArg::Imm(ImmType::Unsigned, ValSize::Size32, imm as u64))), //mem[c]
        Operand::DisplacementU64(imm) => Value::Mem(MemArgs::Mem1Arg(MemArg::Imm(ImmType::Unsigned, ValSize::Size64, imm))), //mem[c]
        Operand::RegDeref(reg) => Value::Mem(MemArgs::Mem1Arg(convert_memarg_reg(reg) )), // mem[reg]
        Operand::RegDisp(reg, imm) => Value::Mem(MemArgs::Mem2Args(convert_memarg_reg(reg), MemArg::Imm(ImmType::Signed, ValSize::Size32, imm as u64)) ), //mem[reg + c]
        Operand::RegIndexBase(reg1, reg2) => Value::Mem(MemArgs::Mem2Args(convert_memarg_reg(reg1), convert_memarg_reg(reg2)) ), // mem[reg1 + reg2]
        Operand::RegIndexBaseDisp(reg1, reg2, imm) => Value::Mem(MemArgs::Mem3Args(convert_memarg_reg(reg1), convert_memarg_reg(reg2), MemArg::Imm(ImmType::Signed, ValSize::Size32, imm as u64)) ), //mem[reg1 + reg2 + c]
        Operand::RegScale(_,_) => panic!("Memory operations with scaling prohibited"), // mem[reg * c]
        Operand::RegScaleDisp(_,_,_) => panic!("Memory operations with scaling prohibited"), //mem[reg*c1 + c2]
        Operand::RegIndexBaseScale(_,_,_) => panic!("Memory operations with scaling prohibited"),//mem[reg1 + reg2*c]
        Operand::RegIndexBaseScaleDisp(_,_,_,_) => panic!("Memory operations with scaling prohibited"),//mem[reg1 + reg2*c1 + c2]
        Operand::Nothing => panic!("Nothing Operand?"),
    }
}




fn clear_reg(instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    Stmt::Clear(instr.opcode, convert_operand(instr.operand(0)))
}

fn clear_dst(instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    Stmt::Clear(instr.opcode, convert_operand(instr.operand(0)))
}

fn unop(instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    Stmt::Unop(instr.opcode, convert_operand(instr.operand(0)), convert_operand(instr.operand(1)))
}

fn binop(instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    Stmt::Binop(instr.opcode, convert_operand(instr.operand(0)), convert_operand(instr.operand(1)), convert_operand(instr.operand(1)))
}

fn branch(instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    Stmt::Branch(instr.opcode, convert_operand(instr.operand(0)))
}

fn call(instr : &yaxpeax_x86::long_mode::Instruction) -> Stmt{
    Stmt::Call(convert_operand(instr.operand(0)))
}

pub fn lift(instr : &yaxpeax_x86::long_mode::Instruction) -> Vec<Stmt>{
    let mut instrs = Vec::new();

    match instr.opcode{
        PUSH => instrs.push(Stmt::Push(convert_operand(instr.operand(0)))), 
        POP => instrs.push(Stmt::Pop(convert_operand(instr.operand(0)))),
        CALL => instrs.push(call(instr)), 
        JO|JNO|JB|JNB|JZ|JNZ|JA|JNA|JS|JNS|JP|JNP|JL|JGE|JLE|JG => instrs.push(branch(instr)),
        TEST => instrs.push(binop(instr)), 
        CMOVA|CMOVB|CMOVG|CMOVGE|CMOVL|CMOVLE|CMOVNA|CMOVNB|
        CMOVNO|CMOVNP|CMOVNS|CMOVNZ|CMOVO|CMOVP|CMOVS|CMOVZ => instrs.push(clear_dst(instr)), 
        SETO|SETNO|SETB|SETAE|SETZ|SETNZ|SETBE|SETA|SETS|
        SETNS|SETP|SETNP|SETL|SETGE|SETLE|SETG => instrs.push(clear_dst(instr)),
        RET => instrs.push(Stmt::Ret), 
        MOVZX => instrs.push(unop(instr)), 
        MOVSXD => instrs.push(unop(instr)),
        ROL => instrs.push(binop(instr)), 
        CMP => instrs.push(binop(instr)),
        MOV => instrs.push(unop(instr)),
        UD2 => instrs.push(Stmt::Undefined),
        LEA => instrs.push(unop(instr)),
        SHL => instrs.push(binop(instr)),
        MOVQ => instrs.push(unop(instr)),
        MOVSD => instrs.push(unop(instr)),
        MOVB => instrs.push(unop(instr)),
        MOVW => instrs.push(unop(instr)),
        AND => instrs.push(binop(instr)), 
        ADD => instrs.push(binop(instr)), 
        RCL => instrs.push(binop(instr)),
        RCR => instrs.push(binop(instr)),
        ROL => instrs.push(binop(instr)),
        ROR => instrs.push(binop(instr)),
        SUB => instrs.push(binop(instr)),
        MOVSX => instrs.push(unop(instr)),
        MOVD => instrs.push(unop(instr)),
        SHR => instrs.push(binop(instr)),
        OR => instrs.push(binop(instr)),
        SUB => instrs.push(binop(instr)),
        JMP => (),

        XOR => instrs.push(clear_dst(instr)),
        SAR => instrs.push(clear_dst(instr)),
        ADC => instrs.push(clear_dst(instr)), 
        XOR => instrs.push(clear_dst(instr)),
        XORB => instrs.push(clear_dst(instr)),
        ROUNDSS => instrs.push(clear_dst(instr)),
        MUL => instrs.push(clear_dst(instr)),
        MOVSS => instrs.push(clear_dst(instr)),
        IMUL => instrs.push(clear_dst(instr)),  
        DIV => instrs.push(clear_dst(instr)), 
        XORPD => instrs.push(clear_dst(instr)),
        MUL => instrs.push(clear_dst(instr)),
        POR => instrs.push(clear_dst(instr)),
        PSHUFB => instrs.push(clear_dst(instr)),
        PSHUFD => instrs.push(clear_dst(instr)),
        PTEST => instrs.push(clear_dst(instr)),
        PXOR => instrs.push(clear_dst(instr)),
        ANDNPS => instrs.push(clear_dst(instr)),  
        XORPS => instrs.push(clear_dst(instr)), 
        XORPD => instrs.push(clear_dst(instr)),
        CMPPD => instrs.push(clear_dst(instr)),
        CMPPS => instrs.push(clear_dst(instr)),
        ANDPS => instrs.push(clear_dst(instr)),
        ORPS => instrs.push(clear_dst(instr)),
        MOVAPS => instrs.push(clear_dst(instr)),
        DIVSD => instrs.push(clear_dst(instr)),
        MULSS => instrs.push(clear_dst(instr)),
        ADDSD => instrs.push(clear_dst(instr)),
        UCOMISD => instrs.push(clear_dst(instr)),
        SUBSS => instrs.push(clear_dst(instr)),
        ROUNDSD => instrs.push(clear_dst(instr)),
        NOT => instrs.push(clear_dst(instr)),
        UCOMISS => instrs.push(clear_dst(instr)),
        POPCNT => instrs.push(clear_dst(instr)),
        SUBSD => instrs.push(clear_dst(instr)),
        MULSD => instrs.push(clear_dst(instr)),
        DIVSS => instrs.push(clear_dst(instr)),
        IDIV => instrs.push(clear_dst(instr)),
        ANDNPS => instrs.push(clear_dst(instr)),
        LZCNT => instrs.push(clear_dst(instr)),
        ANDPS => instrs.push(clear_dst(instr)),
        DIV => instrs.push(clear_dst(instr)),
        DIVPD => instrs.push(clear_dst(instr)),
        DIVPS => instrs.push(clear_dst(instr)),
        DIVSD => instrs.push(clear_dst(instr)),
        DIVSS => instrs.push(clear_dst(instr)),
        IDIV => instrs.push(clear_dst(instr)),
        IMUL => instrs.push(clear_dst(instr)),
        XORPD => instrs.push(clear_dst(instr)),
        XORPS => instrs.push(clear_dst(instr)),
        BLENDVPS => instrs.push(clear_dst(instr)),
        BLENDVPD => instrs.push(clear_dst(instr)),
        MAXPD => instrs.push(clear_dst(instr)),
        MAXPS => instrs.push(clear_dst(instr)),
        MAXSD => instrs.push(clear_dst(instr)),
        MAXSS => instrs.push(clear_dst(instr)),
        MINPD => instrs.push(clear_dst(instr)),
        MINPS => instrs.push(clear_dst(instr)),
        MINSD => instrs.push(clear_dst(instr)),
        MINSS => instrs.push(clear_dst(instr)),
        MULPD => instrs.push(clear_dst(instr)),
        MULPS => instrs.push(clear_dst(instr)),
        MULSD => instrs.push(clear_dst(instr)),
        MULSS => instrs.push(clear_dst(instr)),
        ORPS => instrs.push(clear_dst(instr)),
        PMULLW => instrs.push(clear_dst(instr)),
        PMULLD => instrs.push(clear_dst(instr)),
        PMULLQ => instrs.push(clear_dst(instr)),
        CVTDQ2PS => instrs.push(clear_dst(instr)),
        CVTSD2SS => instrs.push(clear_dst(instr)),
        CVTSI2SD => instrs.push(clear_dst(instr)),
        CVTSI2SS => instrs.push(clear_dst(instr)),
        CVTSS2SD => instrs.push(clear_dst(instr)),
        CVTTSD2SI => instrs.push(clear_dst(instr)),
        CVTTSS2SI => instrs.push(clear_dst(instr)),
        ADDPD => instrs.push(clear_dst(instr)), 
        ADDPS => instrs.push(clear_dst(instr)), 
        ADDSD => instrs.push(clear_dst(instr)), 
        ADDSS => instrs.push(clear_dst(instr)),
        PSLLW => instrs.push(clear_dst(instr)),
        PSLLD => instrs.push(clear_dst(instr)),
        PSLLQ => instrs.push(clear_dst(instr)),
        PSRLW => instrs.push(clear_dst(instr)),
        PSRLD => instrs.push(clear_dst(instr)),
        PSRLQ => instrs.push(clear_dst(instr)),
        PSRAW => instrs.push(clear_dst(instr)),
        PSRAD => instrs.push(clear_dst(instr)),
        PSUBB => instrs.push(clear_dst(instr)),
        PSUBW => instrs.push(clear_dst(instr)),
        PSUBD => instrs.push(clear_dst(instr)),
        PSUBQ => instrs.push(clear_dst(instr)),
        PSUBSB => instrs.push(clear_dst(instr)),
        PSUBSW => instrs.push(clear_dst(instr)),
        PSUBUSB => instrs.push(clear_dst(instr)),
        PSUBUSW => instrs.push(clear_dst(instr)),
        PUNPCKHBW => instrs.push(clear_dst(instr)),
        PUNPCKHWD => instrs.push(clear_dst(instr)),
        PUNPCKHDQ => instrs.push(clear_dst(instr)),
        PUNPCKHQDQ => instrs.push(clear_dst(instr)),
        PUNPCKLBW => instrs.push(clear_dst(instr)),
        PUNPCKLWD => instrs.push(clear_dst(instr)),
        PUNPCKLDQ => instrs.push(clear_dst(instr)),
        PUNPCKLQDQ => instrs.push(clear_dst(instr)),
        PACKSSWB => instrs.push(clear_dst(instr)),
        PACKSSDW => instrs.push(clear_dst(instr)),
        PADDB => instrs.push(clear_dst(instr)),
        PADDD => instrs.push(clear_dst(instr)),
        PADDQ => instrs.push(clear_dst(instr)),
        PADDW => instrs.push(clear_dst(instr)),
        PADDSB => instrs.push(clear_dst(instr)),
        PADDSW => instrs.push(clear_dst(instr)),
        PADDUSB => instrs.push(clear_dst(instr)),
        PADDUSW => instrs.push(clear_dst(instr)),
        PAND => instrs.push(clear_dst(instr)),    
        PANDN => instrs.push(clear_dst(instr)),
        PAVGB => instrs.push(clear_dst(instr)),
        PAVGW => instrs.push(clear_dst(instr)),
        PBLENDVB => instrs.push(clear_dst(instr)),
        PCMPEQB => instrs.push(clear_dst(instr)),
        PCMPEQD => instrs.push(clear_dst(instr)),
        PCMPEQQ => instrs.push(clear_dst(instr)),
        PCMPEQW => instrs.push(clear_dst(instr)),
        PCMPGTB => instrs.push(clear_dst(instr)),
        PCMPGTD => instrs.push(clear_dst(instr)),
        PCMPGTQ => instrs.push(clear_dst(instr)),
        PCMPGTW => instrs.push(clear_dst(instr)),
        PEXTR => instrs.push(clear_dst(instr)),
        PEXTRB => instrs.push(clear_dst(instr)),
        PEXTRW => instrs.push(clear_dst(instr)),
        PINSR => instrs.push(clear_dst(instr)),
        PINSRB => instrs.push(clear_dst(instr)),
        PINSRW => instrs.push(clear_dst(instr)),
        PMAXSB => instrs.push(clear_dst(instr)),
        PMAXSD => instrs.push(clear_dst(instr)),
        PMAXSW => instrs.push(clear_dst(instr)),
        PMAXUB => instrs.push(clear_dst(instr)),
        PMAXUD => instrs.push(clear_dst(instr)),
        PMAXUW => instrs.push(clear_dst(instr)),
        PMINSB => instrs.push(clear_dst(instr)),
        PMINSD => instrs.push(clear_dst(instr)),
        PMINSW => instrs.push(clear_dst(instr)),
        PMINUB => instrs.push(clear_dst(instr)),
        PMINUD => instrs.push(clear_dst(instr)),
        PMINUW => instrs.push(clear_dst(instr)),
        PMOVSXBW => instrs.push(clear_dst(instr)),
        PMOVSXWD => instrs.push(clear_dst(instr)),
        PMOVSXDQ => instrs.push(clear_dst(instr)),
        PMOVZXBW => instrs.push(clear_dst(instr)),
        PMOVZXWD => instrs.push(clear_dst(instr)),
        PMOVZXDQ => instrs.push(clear_dst(instr)),
        SQRTPD => instrs.push(clear_dst(instr)),
        SQRTPS => instrs.push(clear_dst(instr)),
        SQRTSD => instrs.push(clear_dst(instr)),
        SQRTSS => instrs.push(clear_dst(instr)),
        MOVLPS => instrs.push(clear_dst(instr)),
        MOVAPS => instrs.push(clear_dst(instr)),
        MOVLHPS => instrs.push(clear_dst(instr)),
        MOVUPS => instrs.push(clear_dst(instr)),
        SUBPD => instrs.push(clear_dst(instr)),
        SUBPS => instrs.push(clear_dst(instr)),
        SUBSD => instrs.push(clear_dst(instr)),
        SUBSS => instrs.push(clear_dst(instr)),
        TZCNT => instrs.push(clear_dst(instr)),
        SBB => instrs.push(clear_dst(instr)),
        BSR => instrs.push(clear_dst(instr)),
        BSF => instrs.push(clear_dst(instr)),
        UMULX => instrs.push(clear_dst(instr)),
        SMULX => instrs.push(clear_dst(instr)),
        UREM => instrs.push(clear_dst(instr)),
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
            let ir = (addr,lift(instr));
            block_ir.push(ir);
        }
        irmap.insert(block_addr, block_ir);
    };
    irmap
}

