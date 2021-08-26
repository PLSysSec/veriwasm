use crate::{analyses, checkers, ir, lattices, loaders};
use analyses::{AbstractAnalyzer, AnalysisResult, WasmtimeAnalyzer};
use checkers::Checker;
use ir::types::{IRMap, MemArg, MemArgs, RegT, Stmt, ValSize, Value, X86Regs};
use ir::utils::is_stack_access;
use lattices::reachingdefslattice::LocIdx;
use lattices::wasmtime_lattice::{FieldDesc, WasmtimeLattice, WasmtimeValue, WasmtimeValueLattice};
use lattices::VarState;
use std::collections::HashMap;

use FieldDesc::*;
use ValSize::*;
use WasmtimeValue::*;

pub struct WasmtimeChecker<'a, Ar: RegT> {
    irmap: &'a IRMap<Ar>,
    analyzer: &'a WasmtimeAnalyzer,
}

pub fn check_wasmtime<Ar: RegT>(
    result: AnalysisResult<WasmtimeLattice<Ar>>,
    irmap: &IRMap<Ar>,
    analyzer: &WasmtimeAnalyzer,
) -> bool {
    WasmtimeChecker {
        irmap: irmap,
        analyzer: analyzer,
    }
    .check(result)
}

fn is_frame_access<Ar: RegT>(v: &Value<Ar>) -> bool {
    if let Value::Mem(_, memargs) = v {
        // Accept only operands of the form `[rbp + OFFSET]` where `OFFSET` is an integer. In
        // Cranelift-generated code from Wasm, there are never arrays or variable-length data in
        // the function frame, so there should never be a computed address (e.g., `[rbp + 4*eax +
        // OFFSET]`).
        match memargs {
            MemArgs::Mem1Arg(memarg) => memarg.is_rbp(),
            MemArgs::Mem2Args(memarg1, memarg2) => memarg1.is_rbp() && memarg2.is_imm(),
            _ => false,
        }
    } else {
        false
    }
}

impl<Ar: RegT> Checker<Ar, WasmtimeLattice<Ar>> for WasmtimeChecker<'_, Ar> {
    fn check(&self, result: AnalysisResult<WasmtimeLattice<Ar>>) -> bool {
        self.check_state_at_statements(result)
    }

    fn irmap(&self) -> &IRMap<Ar> {
        self.irmap
    }
    fn aexec(&self, state: &mut WasmtimeLattice<Ar>, ir_stmt: &Stmt<Ar>, loc: &LocIdx) {
        self.analyzer.aexec(state, ir_stmt, loc)
    }

    fn check_statement(
        &self,
        state: &WasmtimeLattice<Ar>,
        ir_stmt: &Stmt<Ar>,
        loc_idx: &LocIdx,
    ) -> bool {
        match ir_stmt {
            //1. Check that at each call vmctx reg = VmCtx
            // TODO: reenable
            Stmt::Call(target) => {
                match target {
                    Value::Reg(r, Size64) => {
                        let v = state.regs.get_reg(Ar::pinned_vmctx_reg(), Size64).v;
                        let target_v = state.regs.get_reg(*r, Size64).v;
                        log::debug!("Call check: target = {:?} vmctx reg = {:?}", target_v, v);
                        return target_v
                            .map(|t| t.is_field() && t.as_field().unwrap().is_exec())
                            .unwrap_or(false);
                    }
                    Value::Mem(_, _) => {
                        return false;
                    }
                    _ => return true,
                }

                // return true;
                // return v.map(|x| x.is_vmctx()).unwrap_or(false);
            }

            //2. Check that all load and store are safe
            Stmt::Unop(_, dst, src) => {
                if dst.is_mem() && !self.check_mem_access(state, dst, loc_idx, true) {
                    return false;
                }
                //stack read: probestack <= stackgrowth + c < 8K
                if src.is_mem() && !self.check_mem_access(state, src, loc_idx, false) {
                    return false;
                }
            }

            Stmt::Binop(_, dst, src1, src2) => {
                if dst.is_mem() && !self.check_mem_access(state, dst, loc_idx, true) {
                    return false;
                }
                if src1.is_mem() && !self.check_mem_access(state, src1, loc_idx, false) {
                    return false;
                }
                if src2.is_mem() && !self.check_mem_access(state, src2, loc_idx, false) {
                    return false;
                }
            }
            Stmt::Clear(dst, srcs) => {
                if dst.is_mem() && !self.check_mem_access(state, dst, loc_idx, true) {
                    return false;
                }
                for src in srcs {
                    if src.is_mem() && !self.check_mem_access(state, src, loc_idx, false) {
                        return false;
                    }
                }
            }
            _ => (),
        }
        // not a memory access
        true
    }
}

impl<Ar: RegT> WasmtimeChecker<'_, Ar> {
    fn is_heap_access(&self, state: &WasmtimeLattice<Ar>, access: &Value<Ar>) -> bool {
        match access.to_mem() {
            // 1. mem[heapbase]
            MemArgs::Mem1Arg(MemArg::Reg(regnum, Size64)) => {
                let v = state.regs.get_reg(regnum, Size64).v;
                return v.map(|x| x.is_heapbase()).unwrap_or(false);
            }
            // 2. mem[heapbase + bounded4GB]
            MemArgs::Mem2Args(MemArg::Reg(regnum, Size64), memarg2) => {
                let v1 = state.regs.get_reg(regnum, Size64).v;
                if let Some(HeapBase) = v1 {
                    match memarg2 {
                        MemArg::Reg(regnum2, size2) => {
                            let v2 = state.regs.get_reg(regnum2, size2).v;
                            return v2.map(|x| x.is_heapbase()).unwrap_or(false);
                        }
                        MemArg::Imm(_, _, v) => return v >= -0x1000 && v <= 0xffffffff,
                    }
                };
                false
            }
            // mem[HeapBase + Bounded4GB + Bounded4GB] ||
            // mem[Bounded4GB + HeapBase + Bounded4GB]
            MemArgs::Mem3Args(MemArg::Reg(regnum, Size64), memarg2, memarg3)
            | MemArgs::Mem3Args(memarg2, MemArg::Reg(regnum, Size64), memarg3) => {
                let v1 = state.regs.get_reg(regnum, Size64).v;
                if let Some(HeapBase) = v1 {
                    match (memarg2, memarg3) {
                        (MemArg::Reg(regnum2, size2), MemArg::Imm(_, _, v))
                        | (MemArg::Imm(_, _, v), MemArg::Reg(regnum2, size2)) => {
                            if let Some(Bounded4GB(_)) = state.regs.get_reg(regnum2, size2).v {
                                return v <= 0xffffffff;
                            }
                        }
                        _ => (),
                    }
                };
                false
            }
            _ => return false,
        }
    }
    /// 1. mem[vmctx + c] s.t. c is in offsets
    /// 2. mem[VmCtxField] s.t. write && f.write || ~f.write
    /// vmctx and VmCtxField will always be regs
    fn is_vmctx_access(
        &self,
        state: &WasmtimeLattice<Ar>,
        access: &Value<Ar>,
        write: bool,
    ) -> bool {
        match access.to_mem() {
            MemArgs::Mem1Arg(MemArg::Reg(r, Size64)) => {
                if let Some(v) = state.regs.get_reg(r, Size64).v {
                    if v.is_vmctx() {
                        return true;
                    }
                    if let VmAddr(Some(offset)) = v {
                        // if !self.analyzer.offsets.contains_key(offset){
                        //     println!("This offset does not exist = {:?}", offset);
                        // }
                        let field = self.analyzer.offsets[&offset].clone();
                        if (field.is_write() && write) || (!write) {
                            return true;
                        }
                    }
                    if let Ok(field) = v.as_field() {
                        if (field.is_write() && write) || (!write) {
                            return true;
                        }
                    }
                }
            }
            MemArgs::Mem2Args(MemArg::Reg(r, sz), MemArg::Imm(_, _, imm))
            | MemArgs::Mem2Args(MemArg::Imm(_, _, imm), MemArg::Reg(r, sz)) => {
                let val = state.regs.get_reg(r, sz).v;
                if let Some(ref v) = val {
                    //TODO: whitelist which field offsets are acceptable
                    if v.is_vmctx() && self.analyzer.offsets.contains_key(&imm) {
                        return true;
                    }
                    //TODO: whitelist which field offsets are acceptable
                    if v.is_field() {
                        return true;
                    }
                }
            }
            MemArgs::Mem2Args(MemArg::Reg(r1, sz1), MemArg::Reg(r2, sz2)) => {
                let val1 = state.regs.get_reg(r1, sz1).v;
                let val2 = state.regs.get_reg(r2, sz2).v;
                // match (v1,v2) {
                //     Some(),_
                //     _,Some()
                // }

                // if let (Some(ref v1),Some(ref v2)) = (val1,val2) {
                //     //TODO: refine this further
                //     if v1.is_vmctx() || v2.is_vmctx(){
                //         return true;
                //     }
                //     if v1.is_field() || v2.is_field(){
                //         return true;
                //     }
                // }

                if let Some(ref v1) = val1 {
                    return v1.is_vmctx() || v1.is_field();
                }
                if let Some(ref v2) = val2 {
                    return v2.is_vmctx() || v2.is_field();
                }

                //     //TODO: whitelist which field offsets are acceptable
                //     if v.is_vmctx() && self.analyzer.offsets.contains_key(&imm) {
                //         return true;
                //     }
                //     //TODO: whitelist which field offsets are acceptable
                //     if v.is_field() {
                //         return true;
                //     }
                // }
            }
            _ => (),
        }
        false
    }

    fn check_mem_access(
        &self,
        state: &WasmtimeLattice<Ar>,
        access: &Value<Ar>,
        loc_idx: &LocIdx,
        write: bool,
    ) -> bool {
        // Case 1: its a stack access
        if is_stack_access(access) {
            return true;
        }
        // Case 2: it is a frame slot (RBP-based) access
        if is_frame_access(access) {
            return true;
        }
        // Case 4: its a heap access
        if self.is_heap_access(state, access) {
            return true;
        };

        if self.is_vmctx_access(state, access, write) {
            return true;
        }
        // Case 8: its unknown
        log::debug!(
            "None of the memory accesses at {} : 0x{:x}",
            self.analyzer.name,
            loc_idx.addr
        );
        print_mem_access(state, access);
        return false;
    }
}
pub fn memarg_repr<Ar: RegT>(state: &WasmtimeLattice<Ar>, memarg: &MemArg<Ar>) -> String {
    match memarg {
        MemArg::Reg(regnum, size) => {
            format!("{:?}: {:?}", regnum, state.regs.get_reg(*regnum, *size).v)
        }
        MemArg::Imm(_, _, x) => format!("{:?}", x),
    }
}

pub fn print_mem_access<Ar: RegT>(state: &WasmtimeLattice<Ar>, access: &Value<Ar>) {
    if let Value::Mem(_, memargs) = access {
        match memargs {
            MemArgs::Mem1Arg(x) => log::debug!("{:?} => mem[{:?}]", access, memarg_repr(state, x)),
            MemArgs::Mem2Args(x, y) => log::debug!(
                "{:?} => mem[{:?} + {:?}]",
                access,
                memarg_repr(state, x),
                memarg_repr(state, y)
            ),
            MemArgs::Mem3Args(x, y, z) => log::debug!(
                "{:?} => mem[{:?} + {:?} + {:?}]",
                access,
                memarg_repr(state, x),
                memarg_repr(state, y),
                memarg_repr(state, z)
            ),
            MemArgs::MemScale(x, y, z) => log::debug!(
                "{:?} => mem[{:?} + {:?} * {:?}]",
                access,
                memarg_repr(state, x),
                memarg_repr(state, y),
                memarg_repr(state, z)
            ),
        }
    }
}
