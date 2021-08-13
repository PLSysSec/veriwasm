use crate::{analyses, ir, lattices, loaders};
use analyses::{AbstractAnalyzer, AnalysisResult};
use ir::types::{Binopcode, MemArg, MemArgs, RegT, Stmt, Unopcode, ValSize, Value};
use ir::utils::{extract_stack_offset, is_stack_access};
use lattices::reachingdefslattice::LocIdx;
use lattices::wasmtime_lattice::{FieldDesc, WasmtimeLattice, WasmtimeValue, WasmtimeValueLattice};
use lattices::{ConstLattice, VarState};
// use loaders::types::VwMetadata;
use std::collections::HashMap;
use std::default::Default;

use FieldDesc::*;
use ValSize::*;
use WasmtimeValue::*;

type VMOffsets = HashMap<i64, FieldDesc>;

pub struct WasmtimeAnalyzer {
    offsets: VMOffsets,
    //pub metadata: VwMetadata,
}

impl<Ar: RegT> AbstractAnalyzer<Ar, WasmtimeLattice<Ar>> for WasmtimeAnalyzer {
    fn init_state(&self) -> WasmtimeLattice<Ar> {
        let mut result: WasmtimeLattice<Ar> = Default::default();

        result.regs.set_reg(
            Ar::pinned_vmctx_reg(),
            Size64,
            WasmtimeValueLattice::new(VmCtx),
        );
        result
    }

    fn aexec(&self, in_state: &mut WasmtimeLattice<Ar>, ir_instr: &Stmt<Ar>, loc_idx: &LocIdx) {
        match ir_instr {
            Stmt::Clear(dst, _srcs) => in_state.set_to_bot(dst),
            Stmt::Unop(opcode, dst, src) => self.aexec_unop(in_state, opcode, &dst, &src, loc_idx),
            Stmt::Binop(opcode, dst, src1, src2) => {
                self.aexec_binop(in_state, opcode, dst, src1, src2, loc_idx);
                in_state.adjust_stack_offset(opcode, dst, src1, src2)
            }
            Stmt::Call(_) => in_state.on_call(),
            _ => (),
        }
    }
}

impl WasmtimeAnalyzer {
    fn aexec_unop<Ar: RegT>(
        &self,
        in_state: &mut WasmtimeLattice<Ar>,
        opcode: &Unopcode,
        dst: &Value<Ar>,
        src: &Value<Ar>,
        _loc_idx: &LocIdx,
    ) -> () {
        // Any write to a 32-bit register will clear the upper 32 bits of the containing 64-bit
        // register.
        if let &Value::Reg(rd, Size32) = dst {
            in_state.regs.set_reg(
                rd,
                Size64,
                ConstLattice {
                    v: Some(Bounded4GB),
                },
            );
            return;
        }

        match opcode {
            Unopcode::Mov => {
                let v = self.aeval_unop(in_state, src);
                in_state.set(dst, v);
            }
            Unopcode::Movsx => {
                in_state.set(dst, Default::default());
            }
        }
    }

    fn aexec_binop<Ar: RegT>(
        &self,
        in_state: &mut WasmtimeLattice<Ar>,
        opcode: &Binopcode,
        dst: &Value<Ar>,
        src1: &Value<Ar>,
        src2: &Value<Ar>,
        _loc_idx: &LocIdx,
    ) {
        // match opcode {
        //     Binopcode::Add => {
        //         if let (
        //             &Value::Reg(rd, Size64),
        //             &Value::Reg(rs1, Size64),
        //             &Value::Reg(rs2, Size64),
        //         ) = (dst, src1, src2)
        //         {
        //             let rs1_val = in_state.regs.get_reg(rs1, Size64).v;
        //             let rs2_val = in_state.regs.get_reg(rs2, Size64).v;
        //             match (rs1_val, rs2_val) {
        //                 (Some(HeapBase), Some(Bounded4GB)) | (Some(Bounded4GB), Some(HeapBase)) => {
        //                     in_state
        //                         .regs
        //                         .set_reg(rd, Size64, ConstLattice { v: Some(HeapAddr) });
        //                     return;
        //                 }
        //                 _ => {}
        //             }
        //         }
        //     }
        //     _ => {}
        // }

        // Any write to a 32-bit register will clear the upper 32 bits of the containing 64-bit
        // register.
        if let &Value::Reg(rd, Size32) = dst {
            in_state.regs.set_reg(
                rd,
                Size64,
                ConstLattice {
                    v: Some(Bounded4GB),
                },
            );
            return;
        }

        // in_state.set_to_bot(dst);
    }

    fn aeval_unop<Ar: RegT>(
        &self,
        in_state: &WasmtimeLattice<Ar>,
        value: &Value<Ar>,
    ) -> WasmtimeValueLattice {
        match value {
            Value::Mem(memsize, memargs) => {
                // TODO: VmCtx fields
                // if is_globalbase_access(in_state, memargs) {
                //     return HeapValueLattice::new(GlobalsBase);
                // }

                // All fields in vmctx are 8 byte aligned
                if let Some(field) = self.field_deref(in_state, memargs) {
                    return field;
                }

                if is_stack_access(value) {
                    let offset = extract_stack_offset(memargs);
                    let v = in_state.stack.get(offset, memsize.into_bytes());
                    return v;
                }
            }

            Value::Reg(regnum, size) => {
                if size.into_bits() <= 32 {
                    return WasmtimeValueLattice::new(Bounded4GB);
                } else {
                    return in_state.regs.get_reg(*regnum, Size64);
                }
            }

            Value::Imm(_, _, immval) => {
                if (*immval >= 0) && (*immval < (1 << 32)) {
                    return WasmtimeValueLattice::new(Bounded4GB);
                }
            }
            Value::RIPConst => {
                return Default::default();
            }
        }
        Default::default()
    }

    /// Deref a VmCtx field (if this a deref)
    /// None denotes that this is not a field deref
    /// There are three patters of field access:
    /// 1. mem[VmCtx] => HeapBase
    /// 2. mem[VmCtxField] => VmCtxField.deref()
    /// 3. mem[VmCtx + c] => offsets[c]
    pub fn field_deref<Ar: RegT>(
        &self,
        in_state: &WasmtimeLattice<Ar>,
        memargs: &MemArgs<Ar>,
    ) -> Option<WasmtimeValueLattice> {
        match memargs {
            MemArgs::Mem1Arg(arg) if arg.is_reg() => {
                let v = in_state.regs.get_reg(arg.to_reg(), Size64).v?;
                // 1. mem[VmCtx] => HeapBase
                if v.is_vmctx() {
                    return Some(WasmtimeValueLattice::new(HeapBase));
                }
                // 2. mem[VmCtxField] => VmCtxField.deref()
                if v.is_field() {
                    let field = v.as_field().ok()?;
                    return Some(WasmtimeValueLattice::new(VmCtxField(field)));
                }
            }
            MemArgs::Mem2Args(arg1, arg2) if arg1.is_reg() => {
                // 3. mem[VmCtx + c] => offsets[c]
                let v1 = in_state.regs.get_reg(arg1.to_reg(), Size64).v?;
                if v1.is_vmctx() && arg2.is_imm() {
                    let field = self.offsets[&arg2.to_imm()].clone();
                    return Some(WasmtimeValueLattice::new(VmCtxField(field)));
                }
            }
            _ => (),
        }
        None
    }
}
