use crate::lattices::{ConstLattice, VariableState};

type WasmtimeResult<T> = Result<T, &'static str>;

/// Path description of fields in a struct
/// Fields can either be read/write, read/execute, or
/// A ptr to another field (to handle nested structs)
/// Ptrs are implied to be read-only
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FieldDesc {
    Rw,
    Rx,
    Ptr(Box<FieldDesc>),
}

use FieldDesc::*;

impl FieldDesc {
    pub fn is_write(&self) -> bool {
        matches!(self, Rw)
    }

    pub fn is_exec(&self) -> bool {
        matches!(self, Rx)
    }

    pub fn is_ptr(&self) -> bool {
        matches!(self, Ptr(_))
    }

    pub fn deref(&self) -> WasmtimeResult<FieldDesc> {
        if let Ptr(field) = self {
            Ok(*field.clone())
        } else {
            Err("Tried to deref non-ptr field")
        }
    }
}

/// Lattice used for tracking Heap metadata for wasmtime-compiled programs
/// HeapBase = start of base
/// Bounded4GB = any value truncated to 32 bits
/// Valid Heap accesses are therefore of the form mem[HeapBase + Bounded4GB + Bounded4GB]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum WasmtimeValue {
    HeapBase,
    Bounded4GB,
    VmCtx,
    VmCtxField(FieldDesc),
}

use WasmtimeValue::*;

impl WasmtimeValue {
    pub fn is_heapbase(&self) -> bool {
        matches!(self, HeapBase)
    }

    pub fn is_bounded(&self) -> bool {
        matches!(self, Bounded4GB)
    }

    pub fn is_vmctx(&self) -> bool {
        matches!(self, VmCtx)
    }

    pub fn is_field(&self) -> bool {
        matches!(self, VmCtxField(_))
    }

    pub fn as_field(&self) -> WasmtimeResult<FieldDesc> {
        if let VmCtxField(field) = self {
            Ok(field.clone())
        } else {
            Err("Tried to access a non-field lattice as a field")
        }
    }
}

pub type WasmtimeValueLattice = ConstLattice<WasmtimeValue>;

pub type WasmtimeLattice<Ar> = VariableState<Ar, WasmtimeValueLattice>;
