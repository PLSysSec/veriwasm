use crate::analyses::{AbstractAnalyzer, AnalysisResult, HeapAnalyzer};
use crate::checkers::Checker;
use crate::ir::types::{IRMap, MemArg, MemArgs, Stmt, ValSize, Value, X86Regs, RegT};
use crate::ir::utils::{is_mem_access, is_stack_access};
use crate::lattices::heaplattice::{HeapLattice, HeapValue};
use crate::lattices::reachingdefslattice::LocIdx;
use crate::loaders::utils::is_libcall;
use std::collections::HashMap;

use HeapValue::*;
use ValSize::*;
use X86Regs::*;

pub struct HeapChecker<'a, Ar: RegT> {
    irmap: &'a IRMap<Ar>,
    analyzer: &'a HeapAnalyzer,
    name_addr_map: &'a HashMap<u64, String>,
}

pub fn check_heap<Ar: RegT>(
    result: AnalysisResult<HeapLattice<Ar>>,
    irmap: &IRMap<Ar>,
    analyzer: &HeapAnalyzer,
    name_addr_map: &HashMap<u64, String>,
) -> bool {
    HeapChecker {
        irmap: irmap,
        analyzer: analyzer,
        name_addr_map: name_addr_map,
    }
    .check(result)
}

// fn memarg_is_frame<Ar: RegT>(memarg: &MemArg<Ar>) -> bool {
//     if let MemArg::Reg(Rbp, size) = memarg {
//         assert_eq!(*size, Size64);
//         true
//     } else {
//         false
//     }
// }

fn is_frame_access<Ar: RegT>(v: &Value<Ar>) -> bool {
    if let Value::Mem(_, memargs) = v {
        // Accept only operands of the form `[rbp + OFFSET]` where `OFFSET` is an integer. In
        // Cranelift-generated code from Wasm, there are never arrays or variable-length data in
        // the function frame, so there should never be a computed address (e.g., `[rbp + 4*eax +
        // OFFSET]`).
        match memargs {
            MemArgs::Mem1Arg(memarg) => memarg.is_rbp(),
            MemArgs::Mem2Args(memarg1, memarg2) => {
                memarg1.is_rbp() && matches!(memarg2, MemArg::Imm(..))
            }
            _ => false,
        }
    } else {
        false
    }
}

impl<Ar: RegT> Checker<Ar, HeapLattice<Ar>> for HeapChecker<'_, Ar> {
    fn check(&self, result: AnalysisResult<HeapLattice<Ar>>) -> bool {
        self.check_state_at_statements(result)
    }

    fn irmap(&self) -> &IRMap<Ar> {
        self.irmap
    }
    fn aexec(&self, state: &mut HeapLattice<Ar>, ir_stmt: &Stmt<Ar>, loc: &LocIdx) {
        self.analyzer.aexec(state, ir_stmt, loc)
    }

    fn check_statement(&self, state: &HeapLattice<Ar>, ir_stmt: &Stmt<Ar>, loc_idx: &LocIdx) -> bool {
        match ir_stmt {
            //1. Check that at each call rdi = HeapBase
            Stmt::Call(v) => match state.regs.get_reg(Rdi, Size64).v {
                Some(HeapBase) => (),
                _ => {
                    if let Value::Imm(_, _, dst) = v {
                        let target = (*dst + (loc_idx.addr as i64) + 5) as u64;
                        let name = self.name_addr_map.get(&target).unwrap();
                        if !is_libcall(name) {
                            log::debug!("0x{:x}: Call failure", loc_idx.addr);
                            return false;
                        }
                    } else {
                        log::debug!("0x{:x}: Call failure", loc_idx.addr);
                        return false;
                    }
                }
            },
            //2. Check that all load and store are safe
            Stmt::Unop(_, dst, src) => {
                if is_mem_access(dst) && !self.check_mem_access(state, dst, loc_idx) {
                    return false;
                }
                //stack read: probestack <= stackgrowth + c < 8K
                if is_mem_access(src) && !self.check_mem_access(state, src, loc_idx) {
                    return false;
                }
            }

            Stmt::Binop(_, dst, src1, src2) => {
                if is_mem_access(dst) && !self.check_mem_access(state, dst, loc_idx) {
                    return false;
                }
                if is_mem_access(src1) && !self.check_mem_access(state, src1, loc_idx) {
                    return false;
                }
                if is_mem_access(src2) && !self.check_mem_access(state, src2, loc_idx) {
                    return false;
                }
            }
            Stmt::Clear(dst, srcs) => {
                if is_mem_access(dst) && !self.check_mem_access(state, dst, loc_idx) {
                    return false;
                }
                for src in srcs {
                    if is_mem_access(src) && !self.check_mem_access(state, src, loc_idx) {
                        return false;
                    }
                }
            }
            _ => (),
        }
        true
    }
}

impl<Ar: RegT> HeapChecker<'_, Ar> {
    fn check_global_access(&self, state: &HeapLattice<Ar>, access: &Value<Ar>) -> bool {
        if let Value::Mem(_, memargs) = access {
            match memargs {
                MemArgs::Mem1Arg(MemArg::Reg(regnum, Size64)) => {
                    if let Some(GlobalsBase) = state.regs.get_reg(*regnum, Size64).v {
                        return true;
                    }
                }
                MemArgs::Mem2Args(
                    MemArg::Reg(regnum, Size64),
                    MemArg::Imm(_, _, globals_offset),
                ) => {
                    if let Some(GlobalsBase) = state.regs.get_reg(*regnum, Size64).v {
                        return *globals_offset <= 4096;
                    }
                }
                _ => return false,
            }
        }
        false
    }

    fn check_ripconst_access(&self, state: &HeapLattice<Ar>, access: &Value<Ar>) -> bool {
        if let Value::Mem(_, memargs) = access {
            match memargs {
                // `RIPConst` represents a trusted value laoded from .rodata or .data; any access involving
                // such a pointer is trusted.
                //
                // An offset from the base, even with a computed value,
                // is acceptable here:
                //
                // - If we are checking offline, in a mode where we have access
                //   to symbols/relocations, we will specially recognize table
                //   accesses and they will not reach here.
                //
                // - On the other hand, when we check online, as part of the
                //   compilation and one function at a time without access to
                //   relocations, we accept this approximation to the trusted
                //   base: we trust any memory access based at such a
                //   constant/global-variable-produced address.
                MemArgs::Mem1Arg(MemArg::Reg(regnum, Size64))
                | MemArgs::Mem2Args(MemArg::Reg(regnum, Size64), _)
                | MemArgs::Mem3Args(MemArg::Reg(regnum, Size64), _, _)
                | MemArgs::MemScale(MemArg::Reg(regnum, Size64), _, _) => {
                    if let Some(RIPConst) = state.regs.get_reg(*regnum, Size64).v {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn check_heap_access(&self, state: &HeapLattice<Ar>, access: &Value<Ar>) -> bool {
        if let Value::Mem(_, memargs) = access {
            match memargs {
                // if only arg is heapbase or heapaddr
                MemArgs::Mem1Arg(MemArg::Reg(regnum, Size64)) => {
                    if let Some(HeapBase) = state.regs.get_reg(*regnum, Size64).v {
                        return true;
                    }
                    if let Some(HeapAddr) = state.regs.get_reg(*regnum, Size64).v {
                        return true;
                    }
                }
                // if arg1 is heapbase and arg2 is bounded ||
                // if arg1 is heapaddr and arg2 is constant offset
                MemArgs::Mem2Args(MemArg::Reg(regnum, Size64), memarg2) => {
                    if let Some(HeapBase) = state.regs.get_reg(*regnum, Size64).v {
                        match memarg2 {
                            MemArg::Reg(regnum2, size2) => {
                                if let Some(Bounded4GB) = state.regs.get_reg(*regnum2, *size2).v {
                                    return true;
                                }
                            }
                            MemArg::Imm(_, _, v) => return *v >= -0x1000 && *v <= 0xffffffff,
                        }
                    }
                    if let Some(HeapAddr) = state.regs.get_reg(*regnum, Size64).v {
                        match memarg2 {
                            MemArg::Imm(_, _, v) => return *v >= -0x1000 && *v <= 0xffffffff,
                            _ => {}
                        }
                    }
                }
                // if arg1 is heapbase and arg2 and arg3 are bounded ||
                // if arg1 is bounded and arg1 and arg3 are bounded
                MemArgs::Mem3Args(MemArg::Reg(regnum, Size64), memarg2, memarg3)
                | MemArgs::Mem3Args(memarg2, MemArg::Reg(regnum, Size64), memarg3) => {
                    if let Some(HeapBase) = state.regs.get_reg(*regnum, Size64).v {
                        match (memarg2, memarg3) {
                            (MemArg::Reg(regnum2, size2), MemArg::Imm(_, _, v))
                            | (MemArg::Imm(_, _, v), MemArg::Reg(regnum2, size2)) => {
                                if let Some(Bounded4GB) = state.regs.get_reg(*regnum2, *size2).v {
                                    return *v <= 0xffffffff;
                                }
                            }
                            (MemArg::Reg(regnum2, size2), MemArg::Reg(regnum3, size3)) => {
                                if let (Some(Bounded4GB), Some(Bounded4GB)) = (
                                    state.regs.get_reg(*regnum2, *size2).v,
                                    state.regs.get_reg(*regnum3, *size3).v,
                                ) {
                                    return true;
                                }
                            }
                            _ => (),
                        }
                    }
                }
                _ => return false,
            }
        }
        false
    }

    fn check_metadata_access(&self, state: &HeapLattice<Ar>, access: &Value<Ar>) -> bool {
        if let Value::Mem(_size, memargs) = access {
            match memargs {
                //Case 1: mem[globals_base]
                MemArgs::Mem1Arg(MemArg::Reg(regnum, Size64)) => {
                    if let Some(GlobalsBase) = state.regs.get_reg(*regnum, Size64).v {
                        return true;
                    }
                }
                //Case 2: mem[lucet_tables + 8]
                MemArgs::Mem2Args(MemArg::Reg(regnum, Size64), MemArg::Imm(_, _, 8)) => {
                    if let Some(LucetTables) = state.regs.get_reg(*regnum, Size64).v {
                        return true;
                    }
                }
                MemArgs::Mem2Args(MemArg::Reg(regnum1, Size64), MemArg::Reg(regnum2, Size64)) => {
                    if let Some(GuestTable0) = state.regs.get_reg(*regnum1, Size64).v {
                        return true;
                    }
                    if let Some(GuestTable0) = state.regs.get_reg(*regnum2, Size64).v {
                        return true;
                    }
                }
                MemArgs::Mem3Args(
                    MemArg::Reg(regnum1, Size64),
                    MemArg::Reg(regnum2, Size64),
                    MemArg::Imm(_, _, 8),
                ) => {
                    match (
                        state.regs.get_reg(*regnum1, Size64).v,
                        state.regs.get_reg(*regnum2, Size64).v,
                    ) {
                        (Some(GuestTable0), _) => return true,
                        (_, Some(GuestTable0)) => return true,
                        _ => (),
                    }
                }
                _ => return false,
            }
        }
        false
    }

    fn check_jump_table_access(&self, _state: &HeapLattice<Ar>, access: &Value<Ar>) -> bool {
        if let Value::Mem(_size, memargs) = access {
            match memargs {
                MemArgs::MemScale(_, _, MemArg::Imm(_, _, 4)) => return true,
                _ => return false,
            }
        }
        false
    }

    fn check_mem_access(&self, state: &HeapLattice<Ar>, access: &Value<Ar>, loc_idx: &LocIdx) -> bool {
        // Case 1: its a stack access
        if is_stack_access(access) {
            return true;
        }
        // Case 2: it is a frame slot (RBP-based) access
        if is_frame_access(access) {
            return true;
        }
        // Case 3: it is an access based at a constant loaded from
        // program data. We trust the compiler knows what it's doing
        // in such a case. This could also be a globals or table
        // access if we are validating in-process without relocation
        // info.
        if self.check_ripconst_access(state, access) {
            return true;
        }
        // Case 4: its a heap access
        if self.check_heap_access(state, access) {
            return true;
        };
        // Case 5: its a metadata access
        if self.check_metadata_access(state, access) {
            return true;
        };
        // Case 6: its a globals access
        if self.check_global_access(state, access) {
            return true;
        };
        // Case 7: Jump table access
        if self.check_jump_table_access(state, access) {
            return true;
        };
        // Case 8: its unknown
        log::debug!("None of the memory accesses at 0x{:x}", loc_idx.addr);
        print_mem_access(state, access);
        return false;
    }
}

pub fn memarg_repr<Ar: RegT>(state: &HeapLattice<Ar>, memarg: &MemArg<Ar>) -> String {
    match memarg {
        MemArg::Reg(regnum, size) => {
            format!("{:?}: {:?}", regnum, state.regs.get_reg(*regnum, *size).v)
        }
        MemArg::Imm(_, _, x) => format!("{:?}", x),
    }
}

pub fn print_mem_access<Ar: RegT>(state: &HeapLattice<Ar>, access: &Value<Ar>) {
    if let Value::Mem(_, memargs) = access {
        match memargs {
            MemArgs::Mem1Arg(x) => log::debug!("mem[{:?}]", memarg_repr(state, x)),
            MemArgs::Mem2Args(x, y) => log::debug!(
                "mem[{:?} + {:?}]",
                memarg_repr(state, x),
                memarg_repr(state, y)
            ),
            MemArgs::Mem3Args(x, y, z) => log::debug!(
                "mem[{:?} + {:?} + {:?}]",
                memarg_repr(state, x),
                memarg_repr(state, y),
                memarg_repr(state, z)
            ),
            MemArgs::MemScale(x, y, z) => log::debug!(
                "mem[{:?} + {:?} * {:?}]",
                memarg_repr(state, x),
                memarg_repr(state, y),
                memarg_repr(state, z)
            ),
        }
    }
}
