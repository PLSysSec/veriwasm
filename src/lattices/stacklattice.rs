use std::cmp::Ordering;
use crate::lattices::{Lattice};
use std::collections::HashMap;
use std::default::Default;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct StackSlot<T:Lattice + Copy> {
    size: u32,
    value: T,
}

//Currently implemented with hashmap, could also use a vector for a dense map
#[derive(Eq)]
pub struct StackLattice<T:Lattice + Copy>{
    offset: i64,
    map : HashMap<i64, StackSlot<T>>
}

impl<T:Lattice + Copy> StackLattice<T>{
    fn update(&mut self, offset:i64, value:T, size:u32) -> (){
        self.map.insert(self.offset + offset, StackSlot{size : size, value : value});
    }

    fn get(&self, offset:i64, size:u32) -> T {
        match self.map.get( &(self.offset + offset) ){
            Some(stack_slot) => 
                if stack_slot.size == size { stack_slot.value }
                else { Default::default() },
            None => Default::default()
        }
    }

    fn update_stack_offset(&mut self, adjustment:i64) -> () {
        self.offset += adjustment;
    }
}

//TODO: implement partial order
impl<T:Lattice + Copy> PartialOrd for StackLattice<T> {
    fn partial_cmp(&self, other: &StackLattice<T>) -> Option<Ordering> {
        unimplemented!();
    }
}

impl<T:Lattice + Copy> PartialEq for StackLattice<T> {
    fn eq(&self, other: &StackLattice<T>) -> bool {
        (self.map == other.map) && (self.offset == other.offset)
    }
}

//assumes that stack offset is equal in both stack lattices
impl<T:Lattice + Copy> Lattice for StackLattice<T> {
    fn meet(&self, other : Self) -> Self {
        let mut newmap : HashMap <i64, StackSlot<T>> = HashMap::new();
        for (k,v1) in self.map.iter(){
             match other.map.get(k){
                Some(v2) => 
                if v1.size == v2.size {
                    let newslot =  StackSlot {size : v1.size, value : v1.value.meet(v2.value)};
                    newmap.insert(*k, newslot); 
                },
                None => ()
             }
         }

        StackLattice {offset : self.offset, map : newmap}
    }
} 

impl<T:Lattice + Copy> Default for StackLattice<T> {
    fn default() -> Self {
        StackLattice {offset : 0, map :  HashMap::new()}
    }
}

//TODO: properly test stack lattice
#[test]
fn stack_lattice_test() {
   
}
