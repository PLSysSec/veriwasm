use crate::{analyses, ir, lattices, loaders};
use analyses::{AbstractAnalyzer, AnalysisResult};
use ir::types::{Binopcode, MemArg, MemArgs, RegT, Stmt, Unopcode, ValSize, Value};
use ir::utils::{extract_stack_offset, is_stack_access};
use lattices::reachingdefslattice::LocIdx;
use lattices::wasmtime_lattice::{
    FieldDesc, VMOffsets, WasmtimeLattice, WasmtimeValue, WasmtimeValueLattice,
};
use lattices::{ConstLattice, VarState};
use std::collections::HashMap;
use std::default::Default;

use FieldDesc::*;
use ValSize::*;
use WasmtimeValue::*;

pub struct WasmtimeAnalyzer {
    pub offsets: VMOffsets,
    pub name: String,
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
            Stmt::Clear(dst, _srcs) => {
                if let &Value::Reg(rd, Size32) | &Value::Reg(rd, Size16) | &Value::Reg(rd, Size8) =
                    dst
                {
                    in_state
                        .regs
                        .set_reg(rd, Size64, WasmtimeValueLattice::new(Bounded4GB(None)));
                } else {
                    in_state.set_to_bot(dst)
                }
            }
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
                    v: Some(Bounded4GB(None)),
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
        loc_idx: &LocIdx,
    ) {
        // Any write to a 32-bit register will clear the upper 32 bits of the containing 64-bit
        // register.
        if let &Value::Reg(rd, Size32) = dst {
            in_state.regs.set_reg(
                rd,
                Size64,
                ConstLattice {
                    v: Some(Bounded4GB(None)),
                },
            );
            return;
        }

        in_state.set(dst, self.aeval_binop(in_state, opcode, src1, src2, loc_idx));
    }

    pub fn aeval_unop<Ar: RegT>(
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
                    return WasmtimeValueLattice::new(Bounded4GB(None));
                } else {
                    return in_state.regs.get_reg(*regnum, Size64);
                }
            }

            Value::Imm(_, _, immval) => {
                if (*immval >= 0) && (*immval < (1 << 32)) {
                    return WasmtimeValueLattice::new(Bounded4GB(Some(*immval)));
                }
            }
            Value::RIPConst => {
                return Default::default();
            }
        }
        Default::default()
    }

    pub fn aeval_binop<Ar: RegT>(
        &self,
        in_state: &WasmtimeLattice<Ar>,
        opcode: &Binopcode,
        src1: &Value<Ar>,
        src2: &Value<Ar>,
        loc_idx: &LocIdx,
    ) -> WasmtimeValueLattice {
        match opcode {
            Binopcode::Add => self.aeval_add(in_state, src1, src2, loc_idx),
            _ => Default::default(),
        }
    }

    pub fn aeval_add<Ar: RegT>(
        &self,
        in_state: &WasmtimeLattice<Ar>,
        src1: &Value<Ar>,
        src2: &Value<Ar>,
        loc_idx: &LocIdx,
    ) -> WasmtimeValueLattice {
        // match (src1, src2) {
        //     (&Value::Reg(rs1, sz1), &Value::Reg(rs2, sz2)) => {
        //         let rs1_val = in_state.regs.get_reg(rs1, sz1).v;
        //         let rs2_val = in_state.regs.get_reg(rs2, sz2).v;
        //         match (rs1_val, rs2_val) {
        //             (Some(HeapBase), Some(Bounded4GB(_)))
        //             | (Some(Bounded4GB(_)), Some(HeapBase)) => {
        //                 return WasmtimeValueLattice::new(HeapAddr);
        //             }
        //             (Some(VmCtx), Some(Bounded4GB(Some(b))))
        //             | (Some(Bounded4GB(Some(b))), Some(VmCtx)) => {
        //                 return WasmtimeValueLattice::new(VmAddr(Some(b)));
        //             }
        //             _ => Default::default(),
        //         }

        //         // (&Value::Reg(rs1, sz1), &Value::Reg(rs2, sz2)) => {
        //         //     let rs1_val = in_state.regs.get_reg(rs1, sz1).v;
        //         //     let rs2_val = in_state.regs.get_reg(rs2, sz2).v;
        //         //     match (rs1_val, rs2_val) {
        //         //         (Some(HeapBase), Some(Bounded4GB(_)))
        //         //         | (Some(Bounded4GB(_)), Some(HeapBase)) => {
        //         //             return WasmtimeValueLattice::new(HeapAddr);
        //         //         }
        //         //         (Some(VmCtx), Some(Bounded4GB(Some(b))))
        //         //         | (Some(Bounded4GB(Some(b))), Some(VmCtx)) => {
        //         //             return WasmtimeValueLattice::new(VmAddr(Some(b)));
        //         //         }
        //         //         _ => Default::default(),
        //         //     }
        //         // }
        //     }
        //     _ => Default::default(),
        // }
        match (
            self.aeval_unop(in_state, src1).v,
            self.aeval_unop(in_state, src2).v,
        ) {
            (Some(HeapBase), Some(Bounded4GB(_))) | (Some(Bounded4GB(_)), Some(HeapBase)) => {
                return WasmtimeValueLattice::new(HeapAddr);
            }
            (Some(VmCtx), Some(Bounded4GB(Some(b)))) | (Some(Bounded4GB(Some(b))), Some(VmCtx)) => {
                return WasmtimeValueLattice::new(VmAddr(Some(b)));
            }
            _ => Default::default(),
        }
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
                if let VmAddr(Some(offset)) = v {
                    if !self.offsets.contains_key(&offset) {
                        println!("This offset does not exist = {:?}", offset);
                    }
                    let field_v = self.offsets[&offset].clone();
                    return Some(WasmtimeValueLattice::new(field_v));
                    // return Some(WasmtimeValueLattice::new(VmCtxField(field)));
                }
                // 2. mem[VmCtxField] => Read only field
                if v.is_field() {
                    let field = v.as_field().ok()?;
                    let v = if field.is_ptr() {
                        field.deref().ok()?
                    } else {
                        VmCtxField(FieldDesc::R)
                    };
                    return Some(WasmtimeValueLattice::new(v));
                }
            }
            MemArgs::Mem2Args(arg1, arg2) if arg1.is_reg() => {
                // 3. mem[VmCtx + c] => offsets[c]
                let v1 = in_state.regs.get_reg(arg1.to_reg(), Size64).v?;
                if v1.is_vmctx() && arg2.is_imm() {
                    let k = &arg2.to_imm();
                    if !self.offsets.contains_key(k) {
                        println!("VMCtx field not present: {:?}", k);
                    }
                    let field = self.offsets[k].clone();
                    return Some(WasmtimeValueLattice::new(field));
                    // return Some(WasmtimeValueLattice::new(VmCtxField(field)));
                }
            }
            _ => (),
        }
        None
    }
}
