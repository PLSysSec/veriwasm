use yaxpeax_core::analyses::control_flow::VW_CFG;
use crate::analyses::AnalysisResult;
use yaxpeax_core::analyses::control_flow::ControlFlowGraph;
use crate::lattices::reachingdefslattice::{ReachLattice, singleton, LocIdx};
use crate::analyses::{AbstractAnalyzer, run_worklist};
use crate::lifter::{IRMap, Stmt, Binopcode, Unopcode};
use crate::utils::{LucetMetadata};
use crate::lattices::VarState;

//Top level function
pub fn analyze_reaching_defs(cfg : &VW_CFG, irmap : &IRMap, _metadata : LucetMetadata) -> AnalysisResult<ReachLattice>{
    run_worklist(cfg, irmap, &ReachingDefnAnalyzer{})    
}

pub struct ReachingDefnAnalyzer{}

impl AbstractAnalyzer<ReachLattice> for ReachingDefnAnalyzer {
    fn init_state(&self) -> ReachLattice{
        let mut s: ReachLattice = Default::default();
        
        s.regs.rax = singleton(LocIdx{addr: 0xdeadbeef, idx : 0});
        s.regs.rcx = singleton(LocIdx{addr: 0xdeadbeef, idx : 1});
        s.regs.rdx = singleton(LocIdx{addr: 0xdeadbeef, idx : 2});
        s.regs.rbx = singleton(LocIdx{addr: 0xdeadbeef, idx : 3});
        s.regs.rbp = singleton(LocIdx{addr: 0xdeadbeef, idx : 4});
        s.regs.rsi = singleton(LocIdx{addr: 0xdeadbeef, idx : 5});
        s.regs.rdi = singleton(LocIdx{addr: 0xdeadbeef, idx : 6});

        s.regs.r8 = singleton(LocIdx{addr: 0xdeadbeef, idx : 7});
        s.regs.r9 = singleton(LocIdx{addr: 0xdeadbeef, idx : 8});
        s.regs.r10 = singleton(LocIdx{addr: 0xdeadbeef, idx : 9});
        s.regs.r11 = singleton(LocIdx{addr: 0xdeadbeef, idx : 10});
        s.regs.r12 = singleton(LocIdx{addr: 0xdeadbeef, idx : 11});
        s.regs.r13 = singleton(LocIdx{addr: 0xdeadbeef, idx : 12});
        s.regs.r14 = singleton(LocIdx{addr: 0xdeadbeef, idx : 13});
        s.regs.r15 = singleton(LocIdx{addr: 0xdeadbeef, idx : 14});

        s.stack.update(0x10, singleton(LocIdx{addr: 0xdeadbeef, idx : 15}), 8);
        s.stack.update(0x18, singleton(LocIdx{addr: 0xdeadbeef, idx : 16}), 8);
        s.stack.update(0x20, singleton(LocIdx{addr: 0xdeadbeef, idx : 17}), 8);
        s.stack.update(0x28, singleton(LocIdx{addr: 0xdeadbeef, idx : 18}), 8);

        s
    }


    fn aexec(&self, in_state : &mut ReachLattice, ir_instr : &Stmt, loc_idx : &LocIdx) -> () {
        match ir_instr{
            Stmt::Clear(dst) => in_state.set(dst, singleton(loc_idx.clone())),
            Stmt::Unop(Unopcode::Mov, dst, src) =>  {
                if let Some(v) = in_state.get(src){
                    in_state.set(dst, v)
                }
                else{
                    in_state.set(dst, singleton(loc_idx.clone()))
                }
                //in_state.set(dst, singleton(loc_idx.clone()))
            },
            Stmt::Binop(Binopcode::Cmp, dst, src1, src2) =>  {
                //Ignore compare
            },
            Stmt::Binop(Binopcode::Test, dst, src1, src2) =>  {
                //Ignore test
            },
            Stmt::Binop(opcode, dst, src1, src2) =>  {
                in_state.adjust_stack_offset(opcode, dst, src1, src2);  
                in_state.set(dst, singleton(loc_idx.clone()))
            },
            Stmt::Call(_) => in_state.regs.clear_regs(),
            _ => ()
        }
    }
}
