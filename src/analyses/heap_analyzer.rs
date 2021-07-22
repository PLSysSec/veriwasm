use crate::{analyses, ir, lattices, loaders};
use analyses::{AbstractAnalyzer, AnalysisResult};
use ir::types::{Binopcode, MemArg, MemArgs, Unopcode, ValSize, Value, X86Regs, RegT};
use ir::utils::{extract_stack_offset, is_stack_access};
use lattices::heaplattice::{HeapLattice, HeapValue, HeapValueLattice};
use lattices::reachingdefslattice::LocIdx;
use lattices::{ConstLattice, VarState};
use loaders::types::VwMetadata;
use std::default::Default;

use HeapValue::*;
use ValSize::*;
use X86Regs::*;

pub struct HeapAnalyzer {
    pub metadata: VwMetadata,
}

impl<Ar: RegT> AbstractAnalyzer<Ar, HeapLattice<Ar>> for HeapAnalyzer {
    fn init_state(&self) -> HeapLattice<Ar> {
        let mut result: HeapLattice<Ar> = Default::default();
        result
            .regs
            .set_reg(Rdi, Size64, HeapValueLattice::new(HeapBase));
        result
    }

    fn aexec_unop(
        &self,
        in_state: &mut HeapLattice<Ar>,
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

    fn aexec_binop(
        &self,
        in_state: &mut HeapLattice<Ar>,
        opcode: &Binopcode,
        dst: &Value<Ar>,
        src1: &Value<Ar>,
        src2: &Value<Ar>,
        _loc_idx: &LocIdx,
    ) {
        match opcode {
            Binopcode::Add => {
                if let (
                    &Value::Reg(rd, Size64),
                    &Value::Reg(rs1, Size64),
                    &Value::Reg(rs2, Size64),
                ) = (dst, src1, src2)
                {
                    let rs1_val = in_state.regs.get_reg(rs1, Size64).v;
                    let rs2_val = in_state.regs.get_reg(rs2, Size64).v;
                    match (rs1_val, rs2_val) {
                        (Some(HeapBase), Some(Bounded4GB)) | (Some(Bounded4GB), Some(HeapBase)) => {
                            in_state
                                .regs
                                .set_reg(rd, Size64, ConstLattice { v: Some(HeapAddr) });
                            return;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

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

        in_state.set_to_bot(dst);
    }
}

pub fn is_globalbase_access<Ar: RegT>(in_state: &HeapLattice<Ar>, memargs: &MemArgs<Ar>) -> bool {
    if let MemArgs::Mem2Args(arg1, _arg2) = memargs {
        if let MemArg::Reg(regnum, size) = arg1 {
            assert_eq!(size.into_bits(), 64);
            let base = in_state.regs.get_reg(*regnum, *size);
            if let Some(v) = base.v {
                if let HeapBase = v {
                    return true;
                }
            }
        }
    };
    false
}

impl HeapAnalyzer {
    pub fn aeval_unop<Ar: RegT>(&self, in_state: &HeapLattice<Ar>, value: &Value<Ar>) -> HeapValueLattice {
        match value {
            Value::Mem(memsize, memargs) => {
                if is_globalbase_access(in_state, memargs) {
                    return HeapValueLattice::new(GlobalsBase);
                }
                if is_stack_access(value) {
                    let offset = extract_stack_offset(memargs);
                    let v = in_state.stack.get(offset, memsize.into_bytes());
                    return v;
                }
            }

            Value::Reg(regnum, size) => {
                if size.into_bits() <= 32 {
                    return HeapValueLattice::new(Bounded4GB);
                } else {
                    return in_state.regs.get_reg(*regnum, Size64);
                }
            }

            Value::Imm(_, _, immval) => {
                if (*immval as u64) == self.metadata.guest_table_0 {
                    return HeapValueLattice::new(GuestTable0);
                } else if (*immval as u64) == self.metadata.lucet_tables {
                    return HeapValueLattice::new(LucetTables);
                } else if (*immval >= 0) && (*immval < (1 << 32)) {
                    return HeapValueLattice::new(Bounded4GB);
                }
            }

            Value::RIPConst => {
                return HeapValueLattice::new(RIPConst);
            }
        }
        Default::default()
    }
}
