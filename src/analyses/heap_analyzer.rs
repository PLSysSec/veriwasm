use crate::{analyses, ir, lattices, loaders};
use analyses::AbstractAnalyzer;
use ir::types::{Binopcode, MemArg, MemArgs, Unopcode, ValSize, Value};
use ir::utils::{extract_stack_offset, is_stack_access};
use lattices::heaplattice::{HeapLattice, HeapValue, HeapValueLattice};
use lattices::reachingdefslattice::LocIdx;
use lattices::X86Regs::*;
use lattices::{ConstLattice, VarState};
use loaders::types::VwMetadata;
use std::default::Default;

pub struct HeapAnalyzer {
    pub metadata: VwMetadata,
}

impl AbstractAnalyzer<HeapLattice> for HeapAnalyzer {
    fn init_state(&self) -> HeapLattice {
        let mut result: HeapLattice = Default::default();
        result.regs.set_reg(
            Rdi,
            ValSize::Size64,
            HeapValueLattice::new(HeapValue::HeapBase),
        );
        result
    }

    fn aexec_unop(
        &self,
        in_state: &mut HeapLattice,
        opcode: &Unopcode,
        dst: &Value,
        src: &Value,
        _loc_idx: &LocIdx,
    ) -> () {
        // Any write to a 32-bit register will clear the upper 32 bits of the containing 64-bit
        // register.
        if let &Value::Reg(rd, ValSize::Size32) = dst {
            in_state.regs.set_reg_index(
                &rd,
                &ValSize::Size64,
                ConstLattice {
                    v: Some(HeapValue::Bounded4GB),
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
        in_state: &mut HeapLattice,
        opcode: &Binopcode,
        dst: &Value,
        src1: &Value,
        src2: &Value,
        _loc_idx: &LocIdx,
    ) {
        match opcode {
            Binopcode::Add => {
                if let (
                    &Value::Reg(rd, ValSize::Size64),
                    &Value::Reg(rs1, ValSize::Size64),
                    &Value::Reg(rs2, ValSize::Size64),
                ) = (dst, src1, src2)
                {
                    let rs1_val = in_state.regs.get_reg_index(rs1, ValSize::Size64).v;
                    let rs2_val = in_state.regs.get_reg_index(rs2, ValSize::Size64).v;
                    match (rs1_val, rs2_val) {
                        (Some(HeapValue::HeapBase), Some(HeapValue::Bounded4GB))
                        | (Some(HeapValue::Bounded4GB), Some(HeapValue::HeapBase)) => {
                            in_state.regs.set_reg_index(
                                &rd,
                                &ValSize::Size64,
                                ConstLattice {
                                    v: Some(HeapValue::HeapAddr),
                                },
                            );
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
        if let &Value::Reg(rd, ValSize::Size32) = dst {
            in_state.regs.set_reg_index(
                &rd,
                &ValSize::Size64,
                ConstLattice {
                    v: Some(HeapValue::Bounded4GB),
                },
            );
            return;
        }

        in_state.set_to_bot(dst);
    }
}

pub fn is_globalbase_access(in_state: &HeapLattice, memargs: &MemArgs) -> bool {
    if let MemArgs::Mem2Args(arg1, _arg2) = memargs {
        if let MemArg::Reg(regnum, size) = arg1 {
            assert_eq!(size.into_bits(), 64);
            let base = in_state.regs.get_reg_index(*regnum, *size);
            if let Some(v) = base.v {
                if let HeapValue::HeapBase = v {
                    return true;
                }
            }
        }
    };
    false
}

impl HeapAnalyzer {
    pub fn aeval_unop(&self, in_state: &HeapLattice, value: &Value) -> HeapValueLattice {
        match value {
            Value::Mem(memsize, memargs) => {
                if is_globalbase_access(in_state, memargs) {
                    return HeapValueLattice::new(HeapValue::GlobalsBase);
                }
                if is_stack_access(value) {
                    let offset = extract_stack_offset(memargs);
                    let v = in_state.stack.get(offset, memsize.into_bytes());
                    return v;
                }
            }

            Value::Reg(regnum, size) => {
                if let ValSize::SizeOther = size {
                    return Default::default();
                };
                if size.into_bits() <= 32 {
                    return HeapValueLattice::new(HeapValue::Bounded4GB);
                } else {
                    return in_state.regs.get_reg_index(*regnum, ValSize::Size64);
                }
            }

            Value::Imm(_, _, immval) => {
                if (*immval as u64) == self.metadata.guest_table_0 {
                    return HeapValueLattice::new(HeapValue::GuestTable0);
                } else if (*immval as u64) == self.metadata.lucet_tables {
                    return HeapValueLattice::new(HeapValue::LucetTables);
                } else if (*immval >= 0) && (*immval < (1 << 32)) {
                    return HeapValueLattice::new(HeapValue::Bounded4GB);
                }
            }

            Value::RIPConst => {
                return HeapValueLattice::new(HeapValue::RIPConst);
            }
        }
        Default::default()
    }
}
