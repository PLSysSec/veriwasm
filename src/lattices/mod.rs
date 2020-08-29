use crate::ir_utils::is_rsp;
use crate::ir_utils::get_imm_offset;
use std::cmp::Ordering;
pub mod regslattice;
pub mod heaplattice;
pub mod switchlattice;
pub mod davlattice;
pub mod calllattice;
pub mod stackgrowthlattice;
pub mod stacklattice;
pub mod reachingdefslattice;
use crate::lattices::regslattice::X86RegsLattice;
use crate::lattices::stacklattice::StackLattice;
use crate::lifter::{Value, MemArgs, MemArg, ImmType};

pub trait Lattice: PartialOrd + Eq + Default {
    fn meet(&self, other : &Self) -> Self;
}

#[derive(Eq, Clone, Copy, Debug)]
pub struct BooleanLattice{
    v: bool
}

impl PartialOrd for BooleanLattice {
    fn partial_cmp(&self, other: &BooleanLattice) -> Option<Ordering> {
        Some(self.v.cmp(&other.v))
    }
}

impl PartialEq for BooleanLattice {
    fn eq(&self, other: &BooleanLattice) -> bool {
        self.v == other.v
    }
}

impl Lattice for BooleanLattice {
    fn meet(&self, other : &Self) -> Self {
        BooleanLattice {v : self.v && other.v}
    }
} 

impl Default for BooleanLattice {
    fn default() -> Self {
        BooleanLattice {v : false}
    }
}

pub type Constu32Lattice = ConstLattice::<u32>;

#[derive(Eq, Clone, Copy)]
pub struct ConstLattice<T:Eq + Copy>{
    pub v: Option<T>
}

impl<T:Eq + Copy> PartialOrd for ConstLattice<T> {
    fn partial_cmp(&self, other: &ConstLattice<T>) -> Option<Ordering> {
        match (self.v, other.v){
            (None,None) => Some(Ordering::Equal),
            (None,_) => Some(Ordering::Less),
            (_,None) => Some(Ordering::Greater),
            (Some(x), Some(y)) => 
                if x == y {Some(Ordering::Equal) }
                else {None}
        }
    }
}

impl<T:Eq + Copy> PartialEq for ConstLattice<T> {
    fn eq(&self, other: &ConstLattice<T>) -> bool {
        self.v == other.v
    }
}

impl<T:Eq + Copy> Lattice for ConstLattice<T> {
    fn meet(&self, other : &Self) -> Self {
        if self.v == other.v {ConstLattice {v : self.v}}
        else {ConstLattice { v : None}}
    }
} 

impl<T:Eq + Copy> Default for ConstLattice<T> {
    fn default() -> Self {
        ConstLattice {v : None}
    }
}




#[derive(PartialEq, Eq, PartialOrd, Default, Clone)]
pub struct VariableState<T:Lattice + Clone>{
    pub regs: X86RegsLattice<T>,
    pub stack: StackLattice<T>,
}

impl<T:Lattice + Clone> Lattice for VariableState<T> {
    fn meet(&self, other : &Self) -> Self {
        VariableState { 
            regs : self.regs.meet(&other.regs), 
            stack : self.stack.meet(&other.stack)
        }
    }
} 

//TODO: complete transition to default aexec
impl<T:Lattice + Clone> VariableState<T>{

    pub fn adjust_stack_offset(&mut self, dst: &Value, src1: &Value, src2: &Value){
        if is_rsp(dst) {
            if is_rsp(src1){ 
                let adjustment = get_imm_offset(src2);
                self.stack.update_stack_offset(adjustment)
            }
            else{ panic!("Illegal RSP write") }
        }        
    }

    

    pub fn set(&mut self, index : &Value, value : T) -> (){
        match index{
            Value::Mem(_, memargs) => match memargs{
                MemArgs::Mem1Arg(arg) => 
                    if let MemArg::Reg(regnum, size) = arg{
                        if *regnum == 4{
                            self.stack.update(0, value, size.to_u32())
                        }
                    },
                MemArgs::Mem2Args(arg1, arg2) => 
                    if let MemArg::Reg(regnum, size) = arg1{
                        if *regnum == 4{
                            if let MemArg::Imm(imm_sign,_,offset) = arg2{
                                if let ImmType::Signed = imm_sign{
                                    assert_eq!(false,true);
                                }
                                self.stack.update(*offset, value, size.to_u32())
                            }
                        }
                    },
                _ => ()
            },
            Value::Reg(regnum,_) => self.regs.set(regnum, value),
            Value::Imm(_,_,_) => panic!("Trying to write to an immediate value"),
        }
    }

    pub fn set_to_bot(&mut self, index : &Value){
        self.set(index, Default::default())
    }

    pub fn default_exec_binop(&mut self, dst: &Value, src1: &Value, src2: &Value){
        self.adjust_stack_offset(dst, src1, src2); 
        self.set_to_bot(dst)
    }
}




#[test]
fn boolean_lattice_test() {
    let x  = BooleanLattice {v : false};
    let y  = BooleanLattice {v : true};
    assert_eq!(x < y, true);
    assert_eq!(x > y, false);
    assert_eq!(x.lt(&y), true);
}

#[test]
fn u32_lattice_test() {
    let x1  = ConstLattice::<u32> {v : Some(1)};
    let x2  = ConstLattice::<u32> {v : Some(1)};
    let y1  = ConstLattice::<u32> {v : Some(2)};
    let y2  = ConstLattice::<u32> {v : Some(2)};

    let z1  = Constu32Lattice {v : Some(3)};
    let z2  = Constu32Lattice {v : Some(3)};

    // let y1  = Constu32Lattice {v : Some(2)};
    // let y2  = Constu32Lattice {v : Some(2)};
    assert_eq!(x1 < y1, false);
    assert_eq!(y1 < x1, false);
    assert_eq!(x1 == x2, true);
    assert_eq!(x1 != x2, false);
    assert_eq!(y2 != x1, true);
    assert_eq!(x1 >= y1, false);
    assert_eq!(x1 > x2, false);
    assert_eq!(x1 >= x2, true);
    assert_eq!(z1 == z2, true);
    assert_eq!(z1 == x1, false);
    assert_eq!(x1.lt(&y1), false);
}
