use std::cmp::Ordering;
use crate::lattices::{Lattice, BooleanLattice};
use std::collections::HashMap;
use std::default::Default;

#[derive(PartialEq, Eq, Clone)]
pub struct StackSlot<T:Lattice + Clone> {
    size: u32,
    value: T,
}

//Currently implemented with hashmap, could also use a vector for a dense map
#[derive(Eq, Clone)]
pub struct StackLattice<T:Lattice + Clone>{
    offset: i64,
    map : HashMap<i64, StackSlot<T>>
}

impl<T:Lattice + Clone> StackLattice<T>{
    fn update(&mut self, offset:i64, value:T, size:u32) -> (){
        //Check if 4 aligned
        if (offset & 3) != 0 {
            panic!("Unsafe: Attempt to store value on the stack on not 4-byte aligned address.");
        }
        //remove overlapping entries
        self.map.remove(&(self.offset +  ((offset + (size as i64) - 1) & 0xfffffffc)));
        if size == 8 {
            self.map.remove(&(self.offset +  ((offset + (size as i64) - 1) & 0xfffffff8)));
        }

        //if value is default, just delete entry map.remove(offset)
        if value == Default::default(){
            self.map.remove(&(self.offset + offset));
        }
        else{
            self.map.insert(self.offset + offset, StackSlot{size : size, value : value});
        }

    }

    fn get(&self, offset:i64, size:u32) -> T {
        match self.map.get( &(self.offset + offset) ){
            Some(stack_slot) => 
                if stack_slot.size == size { stack_slot.value.clone() }
                else { Default::default() },
            None => Default::default()
        }
    }

    fn update_stack_offset(&mut self, adjustment:i64) -> () {
        if (adjustment & 3) != 0 {
            panic!("Unsafe: Attempt to make stack not 4-byte aligned.");
        }
        self.offset += adjustment;
    }
}

//TODO: implement partial order
impl<T:Lattice + Clone> PartialOrd for StackLattice<T> {
    fn partial_cmp(&self, other: &StackLattice<T>) -> Option<Ordering> {
        // if self.offset == other.offset {
        //     return None
        // }
        // for (k,v1) in self.map.iter(){
        //     let v2 = other.map.get(k);

        // }
        // Some(Ordering::Less);
        unimplemented!();
    }
}

impl<T:Lattice + Clone> PartialEq for StackLattice<T> {
    fn eq(&self, other: &StackLattice<T>) -> bool {
        (self.map == other.map) && (self.offset == other.offset)
    }
}

//assumes that stack offset is equal in both stack lattices
impl<T:Lattice + Clone> Lattice for StackLattice<T> {
    fn meet(&self, other : Self) -> Self {
        let mut newmap : HashMap <i64, StackSlot<T>> = HashMap::new();
        for (k,v1) in self.map.iter(){
             match other.map.get(k){
                Some(v2) => 
                if v1.size == v2.size {
                    let newslot =  StackSlot {size : v1.size, value : v1.value.meet(v2.value.clone())};
                    newmap.insert(*k, newslot); 
                },
                None => ()
             }
         }

        StackLattice {offset : self.offset, map : newmap}
    }
} 

impl<T:Lattice + Clone> Default for StackLattice<T> {
    fn default() -> Self {
        StackLattice {offset : 0, map :  HashMap::new()}
    }
}

//TODO: properly test stack lattice (ordering, meet, overlapping entries)
#[test]
fn stack_lattice_test() {
    let mut x1 : StackLattice<BooleanLattice> = Default::default();
    let mut x2 : StackLattice<BooleanLattice> = Default::default();
    assert_eq!(x1 == x2, true);
    
    //check equality with adjusted stack
    x1.update_stack_offset(4);
    x2.update_stack_offset(4);
    assert_eq!(x1 == x2, true);
    
    //check inequality of different stack adjustments
    x1.update_stack_offset(2);
    x2.update_stack_offset(4);
    assert_eq!(x1 == x2, false);
    x1.update_stack_offset(2);
    assert_eq!(x1 == x2, true);

    let y1  = BooleanLattice {v : false};
    let y2  = BooleanLattice {v : false};
    let y3  = BooleanLattice {v : true};

    //check equality with entries added
    x1.update(4, y1, 4);
    x2.update(4, y2, 4);
    assert_eq!(x1 == x2, true);

    //check that different sizes break equality
    x1.update(20, y3, 4);
    x2.update(20, y3, 8);
    assert_eq!(x1 != x2, true);
    
    assert_eq!(x1.get(20, 4) == y3, true);
    // should be false if we access with wrong size
    assert_eq!(x1.get(20, 8) == y3, false);
    assert_eq!(x1.get(20, 8) == y1, true); 

    //empty entry should return default
    assert_eq!(x1.get(64, 8) == y1, true); 
    
}
