use crate::{analyses, checkers, ir, lattices};
use analyses::{AbstractAnalyzer, AnalysisResult, HeapAnalyzer};
use checkers::Checker;
use ir::types::{IRMap, MemArg, MemArgs, Stmt, ValSize, Value};
use ir::utils::{is_mem_access, is_stack_access};
use lattices::heaplattice::{HeapLattice, HeapValue};
use lattices::reachingdefslattice::LocIdx;
use lattices::X86Regs::*;

pub struct HeapChecker<'a> {
    irmap: &'a IRMap,
    analyzer: &'a HeapAnalyzer,
}

pub fn check_heap(
    result: AnalysisResult<HeapLattice>,
    irmap: &IRMap,
    analyzer: &HeapAnalyzer,
) -> bool {
    HeapChecker {
        irmap: irmap,
        analyzer: analyzer,
    }
    .check(result)
}

fn memarg_is_frame(memarg: &MemArg) -> bool {
    if let MemArg::Reg(5, size) = memarg {
        assert_eq!(*size, ValSize::Size64);
        true
    } else {
        false
    }
}

fn is_frame_access(v: &Value) -> bool {
    if let Value::Mem(_, memargs) = v {
        // Accept only operands of the form `[rbp + OFFSET]` where `OFFSET` is an integer. In
        // Cranelift-generated code from Wasm, there are never arrays or variable-length data in
        // the function frame, so there should never be a computed address (e.g., `[rbp + 4*eax +
        // OFFSET]`).
        match memargs {
            MemArgs::Mem1Arg(memarg) => memarg_is_frame(memarg),
            MemArgs::Mem2Args(memarg1, memarg2) => {
                memarg_is_frame(memarg1) && matches!(memarg2, MemArg::Imm(..))
            }
            _ => false,
        }
    } else {
        false
    }
}

impl Checker<HeapLattice> for HeapChecker<'_> {
    fn check(&self, result: AnalysisResult<HeapLattice>) -> bool {
        self.check_state_at_statements(result)
    }

    fn irmap(&self) -> &IRMap {
        self.irmap
    }
    fn aexec(&self, state: &mut HeapLattice, ir_stmt: &Stmt, loc: &LocIdx) {
        self.analyzer.aexec(state, ir_stmt, loc)
    }

    fn check_statement(&self, state: &HeapLattice, ir_stmt: &Stmt, _loc_idx: &LocIdx) -> bool {
        match ir_stmt {
            //1. Check that at each call rdi = HeapBase
            Stmt::Call(_) => match state.regs.get_reg(Rdi, ValSize::Size64).v {
                Some(HeapValue::HeapBase) => (),
                _ => {
                    log::debug!("Call failure {:?}", state.stack.get(0, 8));
                    return false;
                }
            },
            //2. Check that all load and store are safe
            Stmt::Unop(_, dst, src) => {
                if is_mem_access(dst) && !self.check_mem_access(state, dst) {
                    return false;
                }
                //stack read: probestack <= stackgrowth + c < 8K
                if is_mem_access(src) && !self.check_mem_access(state, src) {
                    return false;
                }
            }

            Stmt::Binop(_, dst, src1, src2) => {
                if is_mem_access(dst) && !self.check_mem_access(state, dst) {
                    return false;
                }
                if is_mem_access(src1) && !self.check_mem_access(state, src1) {
                    return false;
                }
                if is_mem_access(src2) && !self.check_mem_access(state, src2) {
                    return false;
                }
            }
            Stmt::Clear(dst, srcs) => {
                if is_mem_access(dst) && !self.check_mem_access(state, dst) {
                    return false;
                }
                for src in srcs {
                    if is_mem_access(src) && !self.check_mem_access(state, src) {
                        return false;
                    }
                }
            }
            _ => (),
        }
        true
    }
}

impl HeapChecker<'_> {
    fn check_global_access(&self, state: &HeapLattice, access: &Value) -> bool {
        if let Value::Mem(_, memargs) = access {
            match memargs {
                MemArgs::Mem1Arg(MemArg::Reg(regnum, ValSize::Size64)) => {
                    if let Some(HeapValue::GlobalsBase) =
                        state.regs.get_reg_index(*regnum, ValSize::Size64).v
                    {
                        return true;
                    }
                }
                MemArgs::Mem2Args(
                    MemArg::Reg(regnum, ValSize::Size64),
                    MemArg::Imm(_, _, globals_offset),
                ) => {
                    if let Some(HeapValue::GlobalsBase) =
                        state.regs.get_reg_index(*regnum, ValSize::Size64).v
                    {
                        return *globals_offset <= 4096;
                    }
                }
                _ => return false,
            }
        }
        false
    }

    fn check_ripconst_access(&self, state: &HeapLattice, access: &Value) -> bool {
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
                MemArgs::Mem1Arg(MemArg::Reg(regnum, ValSize::Size64))
                | MemArgs::Mem2Args(MemArg::Reg(regnum, ValSize::Size64), _)
                | MemArgs::Mem3Args(MemArg::Reg(regnum, ValSize::Size64), _, _)
                | MemArgs::MemScale(MemArg::Reg(regnum, ValSize::Size64), _, _) => {
                    if let Some(HeapValue::RIPConst) =
                        state.regs.get_reg_index(*regnum, ValSize::Size64).v
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn check_heap_access(&self, state: &HeapLattice, access: &Value) -> bool {
        if let Value::Mem(_, memargs) = access {
            match memargs {
                // if only arg is heapbase or heapaddr
                MemArgs::Mem1Arg(MemArg::Reg(regnum, ValSize::Size64)) => {
                    if let Some(HeapValue::HeapBase) =
                        state.regs.get_reg_index(*regnum, ValSize::Size64).v
                    {
                        return true;
                    }
                    if let Some(HeapValue::HeapAddr) =
                        state.regs.get_reg_index(*regnum, ValSize::Size64).v
                    {
                        return true;
                    }
                }
                // if arg1 is heapbase and arg2 is bounded ||
                // if arg1 is heapaddr and arg2 is constant offset
                MemArgs::Mem2Args(MemArg::Reg(regnum, ValSize::Size64), memarg2) => {
                    if let Some(HeapValue::HeapBase) =
                        state.regs.get_reg_index(*regnum, ValSize::Size64).v
                    {
                        match memarg2 {
                            MemArg::Reg(regnum2, size2) => {
                                if let Some(HeapValue::Bounded4GB) =
                                    state.regs.get_reg_index(*regnum2, *size2).v
                                {
                                    return true;
                                }
                            }
                            MemArg::Imm(_, _, v) => return *v >= -0x1000 && *v <= 0xffffffff,
                        }
                    }
                    if let Some(HeapValue::HeapAddr) =
                        state.regs.get_reg_index(*regnum, ValSize::Size64).v
                    {
                        match memarg2 {
                            MemArg::Imm(_, _, v) => return *v >= -0x1000 && *v <= 0xffffffff,
                            _ => {}
                        }
                    }
                }
                // if arg1 is heapbase and arg2 and arg3 are bounded ||
                // if arg1 is bounded and arg1 and arg3 are bounded
                MemArgs::Mem3Args(MemArg::Reg(regnum, ValSize::Size64), memarg2, memarg3)
                | MemArgs::Mem3Args(memarg2, MemArg::Reg(regnum, ValSize::Size64), memarg3) => {
                    if let Some(HeapValue::HeapBase) =
                        state.regs.get_reg_index(*regnum, ValSize::Size64).v
                    {
                        match (memarg2, memarg3) {
                            (MemArg::Reg(regnum2, size2), MemArg::Imm(_, _, v))
                            | (MemArg::Imm(_, _, v), MemArg::Reg(regnum2, size2)) => {
                                if let Some(HeapValue::Bounded4GB) =
                                    state.regs.get_reg_index(*regnum2, *size2).v
                                {
                                    return *v <= 0xffffffff;
                                }
                            }
                            (MemArg::Reg(regnum2, size2), MemArg::Reg(regnum3, size3)) => {
                                if let (Some(HeapValue::Bounded4GB), Some(HeapValue::Bounded4GB)) = (
                                    state.regs.get_reg_index(*regnum2, *size2).v,
                                    state.regs.get_reg_index(*regnum3, *size3).v,
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

    fn check_metadata_access(&self, state: &HeapLattice, access: &Value) -> bool {
        if let Value::Mem(_size, memargs) = access {
            match memargs {
                //Case 1: mem[globals_base]
                MemArgs::Mem1Arg(MemArg::Reg(regnum, ValSize::Size64)) => {
                    if let Some(HeapValue::GlobalsBase) =
                        state.regs.get_reg_index(*regnum, ValSize::Size64).v
                    {
                        return true;
                    }
                }
                //Case 2: mem[lucet_tables + 8]
                MemArgs::Mem2Args(MemArg::Reg(regnum, ValSize::Size64), MemArg::Imm(_, _, 8)) => {
                    if let Some(HeapValue::LucetTables) =
                        state.regs.get_reg_index(*regnum, ValSize::Size64).v
                    {
                        return true;
                    }
                }
                MemArgs::Mem2Args(
                    MemArg::Reg(regnum1, ValSize::Size64),
                    MemArg::Reg(regnum2, ValSize::Size64),
                ) => {
                    if let Some(HeapValue::GuestTable0) =
                        state.regs.get_reg_index(*regnum1, ValSize::Size64).v
                    {
                        return true;
                    }
                    if let Some(HeapValue::GuestTable0) =
                        state.regs.get_reg_index(*regnum2, ValSize::Size64).v
                    {
                        return true;
                    }
                }
                MemArgs::Mem3Args(
                    MemArg::Reg(regnum1, ValSize::Size64),
                    MemArg::Reg(regnum2, ValSize::Size64),
                    MemArg::Imm(_, _, 8),
                ) => {
                    match (
                        state.regs.get_reg_index(*regnum1, ValSize::Size64).v,
                        state.regs.get_reg_index(*regnum2, ValSize::Size64).v,
                    ) {
                        (Some(HeapValue::GuestTable0), _) => return true,
                        (_, Some(HeapValue::GuestTable0)) => return true,
                        _ => (),
                    }
                }
                _ => return false,
            }
        }
        false
    }

    fn check_jump_table_access(&self, _state: &HeapLattice, access: &Value) -> bool {
        if let Value::Mem(_size, memargs) = access {
            match memargs {
                MemArgs::MemScale(_, _, MemArg::Imm(_, _, 4)) => return true,
                _ => return false,
            }
        }
        false
    }

    fn check_mem_access(&self, state: &HeapLattice, access: &Value) -> bool {
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
        log::debug!("None of the memory accesses!");
        print_mem_access(state, access);
        return false;
    }
}

pub fn memarg_repr(state: &HeapLattice, memarg: &MemArg) -> String {
    match memarg {
        MemArg::Reg(regnum, size) => format!(
            "r{:?}: {:?}",
            regnum,
            state.regs.get_reg_index(*regnum, *size).v
        ),
        MemArg::Imm(_, _, x) => format!("{:?}", x),
    }
}

pub fn print_mem_access(state: &HeapLattice, access: &Value) {
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
