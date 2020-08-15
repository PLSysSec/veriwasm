use std::cmp::Ordering;
use crate::lattices::{Lattice, Valued};

pub struct StackSlot {
    size: u32,
    value: i64,
}

//TODO: Currently implemented with vector, could also try with associative map

#[derive(Eq)]
pub struct StackLattice{
    offset: i64,
    map : Vec<Option<StackSlot>>

    fn update(&self, i32 : offset, i64 : value, u32 : size) -> {
        // Index in the vec
        let idx =  self.get_stack_idx(offset);
        // remap with more size if we need it
        self.map.resize_with(idx + 1, Default::default);
        if (idx > 0) && self.map[idx - 1].size == 8 
        {
            self.map[idx - 1] = None;
        }

        if size == 8 {
            self.map[idx + 1] = None;
        }

        self.map[idx] = value;
    }

    fn get(&self, i32 : offset, u32 : size) -> i64 {
        let idx =  self.get_stack_idx(offset);
        self.map.resize_with(idx + 1, Default::default);
        self.map[idx]
    }

    fn get_stack_idx(&self, i64 : offset)
    {
        (self.offset + offset) / 4
    }

    fn update_stack_offset(&self, i64: adjustment) -> {
        self.offset += adjustment;
    }
}

//TODO: implement partial order
impl PartialOrd for StackLattice {
    fn partial_cmp(&self, other: &StackLattice) -> Option<Ordering> {
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

impl PartialEq for StackLattice {
    fn eq(&self, other: &StackLattice) -> bool {
        (self.map == other.map) && (self.offset == other.offset)
    }
}

//TODO: implement meet
impl Lattice for StackLattice {
    fn meet(&self, other : Self) -> Self {
        if self.v == other.v {StackLattice {v : self.v}}
        else {StackLattice { v : None}}
    }
} 

impl Default for StackLattice {
    fn default() -> Self {
        StackLattice {offset : 0, map :  Vec::new()}
    }
}


