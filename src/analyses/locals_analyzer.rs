// use crate::{analyses, ir, lattices, loaders};
// use std::collections::HashMap;
// use std::collections::HashSet;

// use analyses::{AbstractAnalyzer, AnalysisResult, CallAnalyzer};
// use ir::types::{Binopcode, FunType, IRMap, Stmt, ValSize, Value, VarIndex, X86Regs, RegT};
// use ir::utils::mk_value_i64;
// use lattices::calllattice::CallCheckLattice;
// use lattices::localslattice::*;
// use lattices::mem_to_stack_offset;
// use lattices::reachingdefslattice::LocIdx;
// use lattices::{Lattice, VarState, VariableState};
// use loaders::types::VwFuncInfo;
// use loaders::utils::to_system_v_ret_ty;

// use SlotVal::*;
// use ValSize::*;
// use X86Regs::*;

// pub struct LocalsAnalyzer<'a, Ar> {
//     pub fun_type: FunType,
//     pub symbol_table: &'a VwFuncInfo,
//     pub name_addr_map: &'a HashMap<u64, String>,
//     pub plt_bounds: (u64, u64),
//     pub call_analysis: AnalysisResult<CallCheckLattice<Ar>>,
//     pub call_analyzer: CallAnalyzer<Ar>,
// }

// impl<'a, Ar: RegT> LocalsAnalyzer<'a, Ar> {
//     pub fn aeval_val(&self, state: &LocalsLattice<Ar>, value: &Value<Ar>, loc_idx: &LocIdx) -> SlotVal {
//         match value {
//             Value::Mem(memsize, memargs) => {
//                 if let Some(offset) = mem_to_stack_offset(memargs) {
//                     // println!("reading from stack 0x{:x?}: {:?} + {:?}\n\t{:?}", loc_idx.addr, state.stack.offset, offset, value);
//                     // println!("{}", state.stack);
//                     // println!("{:?}", self.fun_type);
//                     state.stack.get(offset, memsize.into_bytes())
//                 } else {
//                     // println!("reading from mem 0x{:x?}: {:?}", loc_idx.addr, value);
//                     Init
//                 }
//             }
//             Value::Reg(_, _) => state.get(value).unwrap_or(Uninit),
//             Value::Imm(_, _, _) => Init,
//             Value::RIPConst => todo!(),
//         }
//     }

//     // if all values are initialized then the value is initialized
//     pub fn aeval_vals(
//         &self,
//         state: &LocalsLattice<Ar>,
//         values: &Vec<Value<Ar>>,
//         loc_idx: &LocIdx,
//     ) -> SlotVal {
//         values.iter().fold(Init, |acc, value| -> SlotVal {
//             if (acc == Init) && (self.aeval_val(state, value, loc_idx) == Init) {
//                 Init
//             } else {
//                 Uninit
//             }
//         })
//     }
// }

// impl<'a, Ar: RegT> AbstractAnalyzer<Ar, LocalsLattice<Ar>> for LocalsAnalyzer<'a, Ar> {
//     fn init_state(&self) -> LocalsLattice<Ar> {
//         let mut lattice: LocalsLattice<Ar> = Default::default();
//         for arg in self.fun_type.args.iter() {
//             match arg {
//                 (VarIndex::Stack(offset), size) => {
//                     lattice
//                         .stack
//                         .update(i64::from(*offset), Init, size.into_bytes())
//                 }
//                 (VarIndex::Reg(reg_num), size) => lattice.regs.set_reg(*reg_num, *size, Init),
//             }
//         }
//         // rbp, rbx, and r12-r15 are the callee-saved registers
//         lattice.regs.set_reg(Rbp, Size64, UninitCalleeReg(Rbp));
//         lattice.regs.set_reg(Rbx, Size64, UninitCalleeReg(Rbx));
//         lattice.regs.set_reg(R12, Size64, UninitCalleeReg(R12));
//         lattice.regs.set_reg(R13, Size64, UninitCalleeReg(R13));
//         lattice.regs.set_reg(R14, Size64, UninitCalleeReg(R14));
//         lattice.regs.set_reg(R15, Size64, UninitCalleeReg(R15));
//         lattice
//     }

//     fn aexec(&self, in_state: &mut LocalsLattice<Ar>, ir_instr: &Stmt<Ar>, loc_idx: &LocIdx) -> () {
//         let debug_addrs: HashSet<u64> = vec![].into_iter().collect();
//         if debug_addrs.contains(&loc_idx.addr) {
//             println!("========================================");
//             println!("{}", in_state);
//             println!("aexec debug 0x{:x?}: {:?}", loc_idx.addr, ir_instr);
//         }
//         match ir_instr {
//             Stmt::Clear(dst, srcs) => in_state.set(dst, self.aeval_vals(in_state, srcs, loc_idx)),
//             Stmt::Unop(_, dst, src) => in_state.set(dst, self.aeval_val(in_state, src, loc_idx)),
//             Stmt::Binop(opcode, dst, src1, src2) => {
//                 let dst_val = self
//                     .aeval_val(in_state, src1, loc_idx)
//                     .meet(&self.aeval_val(in_state, src2, loc_idx), loc_idx);
//                 in_state.set(dst, dst_val);
//                 let prev_offset = in_state.stack.offset;
//                 in_state.adjust_stack_offset(opcode, dst, src1, src2);
//                 // if prev_offset != in_state.stack.offset {
//                 //     println!("adjusting offset 0x{:x?}: {:?}\n\t{:?} -> {:?}",
//                 //              loc_idx.addr,
//                 //              ir_instr,
//                 //              prev_offset,
//                 //              in_state.stack.offset,
//                 //     );
//                 // }
//             }
//             // TODO: wasi calls (no requirements on initialization, always return in rax)
//             // TODO: wasi calls are things in plt_bounds?
//             Stmt::Call(Value::Imm(_, _, dst)) => {
//                 let target = (*dst + (loc_idx.addr as i64) + 5) as u64;
//                 let name = self.name_addr_map.get(&target);
//                 let signature = name
//                     .and_then(|name| self.symbol_table.indexes.get(name))
//                     .and_then(|sig_index| self.symbol_table.signatures.get(*sig_index as usize));
//                 in_state.on_call();
//                 if let Some((ret_reg, reg_size)) = signature.and_then(|sig| to_system_v_ret_ty(sig))
//                 {
//                     in_state.regs.set_reg(ret_reg, reg_size, Init);
//                 }
//             }
//             Stmt::Call(val @ Value::Reg(_, _)) => {
//                 let fn_ptr_type = self
//                     .call_analyzer
//                     .get_fn_ptr_type(&self.call_analysis, loc_idx, val)
//                     .and_then(|fn_ptr_index| {
//                         self.symbol_table.signatures.get(fn_ptr_index as usize)
//                     });
//                 in_state.on_call();
//                 if let Some((ret_reg, reg_size)) =
//                     fn_ptr_type.and_then(|sig| to_system_v_ret_ty(sig))
//                 {
//                     in_state.regs.set_reg(ret_reg, reg_size, Init);
//                 }
//             }
//             Stmt::Branch(br_type, val) => {
//                 // println!("unhandled branch 0x{:x?}: {:?} {:?}", loc_idx.addr, br_type, val);
//             }
//             Stmt::ProbeStack(x) => {
//                 in_state.adjust_stack_offset(
//                     &Binopcode::Sub,
//                     &Value::Reg(Rsp, Size64),
//                     &Value::Reg(Rsp, Size64),
//                     &mk_value_i64(*x as i64),
//                 );
//             }
//             stmt => {
//                 // println!("unhandled instruction 0x{:x?}: {:?}", loc_idx.addr, stmt);
//             }
//         }
//         if debug_addrs.contains(&loc_idx.addr) {
//             println!("{}", in_state);
//             println!("========================================");
//         }
//     }

//     fn process_branch(
//         &self,
//         _irmap: &IRMap<Ar>,
//         in_state: &LocalsLattice<Ar>,
//         succ_addrs: &Vec<u64>,
//         _addr: &u64,
//     ) -> Vec<(u64, LocalsLattice<Ar>)> {
//         succ_addrs
//             .into_iter()
//             .map(|addr| (addr.clone(), in_state.clone()))
//             .collect()
//     }
// }
