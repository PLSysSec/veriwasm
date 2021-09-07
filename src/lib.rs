//! Library entry point for stack and heap validation, given a single
//! function's machine code and basic-block offsets.

#![allow(dead_code, unused_imports, unused_variables)]
pub mod analyses;
pub mod checkers;
pub mod ir;
pub mod lattices;
pub mod loaders;
pub mod runner;

use analyses::run_worklist;
use analyses::{HeapAnalyzer, StackAnalyzer, WasmtimeAnalyzer};
use checkers::{check_heap, check_stack, check_wasmtime};
// use ir::lift_cfg;
use crate::ir::types::X86Regs;
use ir::types::IRMap;
use ir::{aarch64_lift_cfg, x64_lift_cfg};
use lattices::wasmtime_lattice::{FieldDesc, VMOffsets, WasmtimeValue};
use loaders::types::{ExecutableType, VwArch, VwMetadata, VwModule};
use petgraph::graphmap::GraphMap;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::str::FromStr;
use wasmtime_environ;
use yaxpeax_core::analyses::control_flow::{VW_Block, VW_CFG};
use yaxpeax_core::memory::repr::process::{ModuleData, ModuleInfo, Segment};

use crate::WasmtimeValue::*;

#[derive(Clone, Copy, Debug)]
pub enum ValidationError {
    StackUnsafe,
    HeapUnsafe,
    Other(&'static str),
}
impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for ValidationError {}

/// How the Wasm heap is accessed in machine code. This will allow the
/// check to be parameterized to work with different VMs -- first
/// Lucet, eventually Wasmtime, perhaps others -- that have slightly
/// different VM-context data structure layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeapStrategy {
    /// The first argument to functions is a hidden argument that is
    /// the heap base. Accesses to the heap are computed relative to
    /// this base. The virtual-memory layout has sufficient guard
    /// regions that no bounds-checks are necessary as long as only an
    /// unsigned 32-bit offset is added to the base.
    ///
    /// This corresponds to Lucet's design.
    HeapPtrFirstArgWithGuards,

    /// The first argument to functions is a hidden VM-context struct
    /// pointer, and another pointer within this struct points to the
    /// Wasm heap. The guard region is assumed to be present as
    /// above. The offset to the heap-base pointer within vmctx is
    /// configurable.
    ///
    /// This corresponds to Wasmtime's design.
    VMCtxFirstArgWithGuards { vmctx_heap_base_ptr_offset: usize },
}

fn get_cfg_from_compiler_info(
    code: &[u8],
    basic_blocks: &[usize],
    cfg_edges: &[(usize, usize)],
) -> VW_CFG {
    // We build the VW_CFG manually; we skip the CFG-recovery
    // algorithm that has to analyze the machine code and compute
    // reaching-defs in a fixpoint loop.
    let mut cfg = VW_CFG {
        entrypoint: 0,
        blocks: BTreeMap::new(),
        graph: GraphMap::new(),
    };

    for i in 0..basic_blocks.len() {
        let start = basic_blocks[i] as u64;
        let end = if i == basic_blocks.len() - 1 {
            code.len() as u64
        } else {
            basic_blocks[i + 1] as u64
        };
        assert!(end > start, "block has zero length: {} -> {}", start, end);
        let end = end - 1; // `end` is inclusive!
        let bb = VW_Block { start, end };
        cfg.blocks.insert(start, bb);
        cfg.graph.add_node(start);
    }
    for &(from, to) in cfg_edges {
        cfg.graph.add_edge(from as u64, to as u64, ());
    }

    cfg
}

fn create_dummy_module(code: &[u8], format: ExecutableType, arch: VwArch) -> VwModule {
    let seg = Segment {
        start: 0,
        data: code.to_vec(),
        name: ".text".to_owned(),
    };
    let header = yaxpeax_core::goblin::elf::header::Header {
        e_ident: [
            0x7f, 0x45, 0x4c, 0x4f, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ],
        e_type: 0x0003,
        e_machine: 0x003e,
        e_version: 0x00000001,
        e_entry: 0,
        e_phoff: 0,
        e_shoff: 0,
        e_flags: 0,
        e_ehsize: 0,
        e_phentsize: 0,
        e_phnum: 0,
        e_shentsize: 0,
        e_shnum: 0,
        e_shstrndx: 0,
    };
    let module_info = ModuleInfo::ELF(
        yaxpeax_core::memory::repr::process::ISAHint::Hint(yaxpeax_core::arch::ISA::x86_64),
        header,
        vec![],
        vec![],
        0,
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let data = ModuleData {
        segments: vec![seg],
        name: "function.o".to_owned(),
        module_info,
    };
    let lucet = VwMetadata {
        guest_table_0: 0x123456789abcdef0,
        lucet_tables: 0x123456789abcdef0,
        lucet_probestack: 0x123456789abcdef0,
    };

    let module = VwModule {
        program: data,
        metadata: lucet,
        format: format,
        arch: arch,
    };

    module
}

fn create_dummy_lucet_module(code: &[u8]) -> VwModule {
    create_dummy_module(code, ExecutableType::Lucet, VwArch::X64)
}

pub fn validate_heap(
    code: &[u8],
    basic_blocks: &[usize],
    cfg_edges: &[(usize, usize)],
    heap_strategy: HeapStrategy,
) -> Result<(), ValidationError> {
    log::debug!(
        "validate_heap: basic_blocks = {:?}, edges = {:?}",
        basic_blocks,
        cfg_edges
    );
    // For now, we don't support Wasmtime-style heap accesses.
    // TODO: implement these:
    // - Add a lattice value: VMCtxPtr
    // - Add a rule: load from [VMCtxPtr + heap_base_ptr_offset] -> HeapBase
    match heap_strategy {
        HeapStrategy::HeapPtrFirstArgWithGuards => {}
        _ => {
            log::debug!("Unknown heap strategy: {:?}", heap_strategy);
            return Err(ValidationError::HeapUnsafe);
        }
    }

    let cfg = get_cfg_from_compiler_info(code, basic_blocks, cfg_edges);
    let module = create_dummy_lucet_module(&code);
    let irmap: IRMap<X86Regs> = x64_lift_cfg(&module, &cfg, false);

    // TODO: regalloc checker from Lucet too.
    // TODO: how hard would this be to adapt to Wasmtime? Extra level of indirection:
    //       heap-base loaded from vmctx (rdi) instead. Take a mode argument?
    //       "Heap-access style"

    // This entry point is designed to allow checking of a single
    // function body, just after it has been generated in memory,
    // without the metadata that would usually come along with an ELF
    // binary.
    //
    // Without symbols/relocs, we don't know which calls are to
    // `lucet_probestack()`, so we can't do a stack-use soundness
    // check, and we don't know which globals are `lucet_tables` and
    // `guest_table_0`, so we can't check instance function calls. We
    // also can't really do the full CFG recovery analysis and CFI
    // checks because it's very expensive (the reaching-defs analysis
    // has not been optimized) and requires knowing other function
    // addresses.
    //
    // However, the heap check is the most important one, and we *can*
    // do that. Why are the others less important? Mainly because we
    // trust their implementations a little more: e.g., the br_table
    // code is a single open-coded sequence that is generated just at
    // machine-code emission, after all optimizations and regalloc,
    // with its bounds-check embedded inside. The CFG lowering itself
    // is handled by the MachBuffer in the new Cranelift backend, and
    // this has a correctness proof. Stack probes are either present
    // or not, and we have tests to ensure that they are when the
    // frame is large enough. The address computation that goes into a
    // heap access is the most exposed -- it's just ordinary CLIF IR
    // that goes through the compilation pipeline with opt passes like
    // all other code. It's also the fastest and simplest to check.
    let heap_analyzer = HeapAnalyzer {
        metadata: module.metadata,
    };
    let heap_result = run_worklist(&cfg, &irmap, &heap_analyzer);
    // let heap_safe = check_heap(heap_result, &irmap, &heap_analyzer);
    // if !heap_safe {
    //     return Err(ValidationError::HeapUnsafe);
    // }

    Ok(())
}

pub fn wasmtime_test_hook() {
    println!("Wasmtime has called into VeriWasm!");
}

/*
VMOffsets {
    pointer_size: 8,
    num_signature_ids: 51,
    num_imported_functions: 20,
    num_imported_tables: 0,
    num_imported_memories: 0,
    num_imported_globals: 0,
    num_defined_functions: 731,
    num_defined_tables: 1,
    num_defined_memories: 1,
    num_defined_globals: 243,
    interrupts: 0,
    externref_activations_table: 8,
    module_info_lookup: 16,
    signature_ids: 32,
    imported_functions: 236,
    imported_tables: 556,
    imported_memories: 556,
    imported_globals: 556,
    defined_tables: 556,
    defined_memories: 572,
    defined_globals: 592,
    defined_anyfuncs: 4480,
    builtin_functions: 22504,
    size: 22720 }

*/

//
// vmcaller_checked_anyfunc_func_ptr 0    ?
// vmcaller_checked_anyfunc_type_index 8  ?
// vmcaller_checked_anyfunc_vmctx 16      ?
// size_of_vmcaller_checked_anyfunc() 24  ?
// vmctx_interrupts 0                     ?
// vmctx_externref_activations_table 8    ?
// vmctx_module_info_lookup 16            ?
// vmctx_signature_ids_begin 32           Rx
// vmctx_imported_functions_begin 236     Rx
// vmctx_imported_tables_begin 556        R
// vmctx_imported_memories_begin 556      R
// vmctx_imported_globals_begin  556      R
// vmctx_tables_begin 556                 R
// vmctx_memories_begin 572               address of HeapBase
// vmctx_globals_begin 592                R
// vmctx_anyfuncs_begin 4480              ?
// vmctx_builtin_functions_begin 22504    ?
// size_of_vmctx 22720                    ?
//

fn get_vm_offsets(vm_offsets: &wasmtime_environ::VMOffsets) -> VMOffsets {
    // println!("VMOffsets:
    //     {:?}
    //     vmcaller_checked_anyfunc_func_ptr {:?}
    //     vmcaller_checked_anyfunc_type_index {:?}
    //     vmcaller_checked_anyfunc_vmctx {:?}
    //     size_of_vmcaller_checked_anyfunc() {:?}
    //     vmctx_interrupts {:?}
    //     vmctx_externref_activations_table {:?}
    //     vmctx_module_info_lookup {:?}
    //     vmctx_signature_ids_begin {:?}
    //     vmctx_imported_functions_begin {:?}
    //     vmctx_imported_tables_begin {:?}
    //     vmctx_imported_memories_begin {:?}
    //     vmctx_imported_globals_begin  {:?}
    //     vmctx_tables_begin {:?}
    //     vmctx_memories_begin {:?}
    //     vmctx_globals_begin {:?}
    //     vmctx_anyfuncs_begin {:?}
    //     vmctx_builtin_functions_begin {:?}
    //     size_of_vmctx {:?}",
    //     vm_offsets,
    //     vm_offsets.vmcaller_checked_anyfunc_func_ptr(),
    //     vm_offsets.vmcaller_checked_anyfunc_type_index(),
    //     vm_offsets.vmcaller_checked_anyfunc_vmctx(),
    //     vm_offsets.size_of_vmcaller_checked_anyfunc(),
    //     vm_offsets.vmctx_interrupts(),
    //     vm_offsets.vmctx_externref_activations_table(),
    //     vm_offsets.vmctx_module_info_lookup(),
    //     vm_offsets.vmctx_signature_ids_begin(),
    //     vm_offsets.vmctx_imported_functions_begin(),
    //     vm_offsets.vmctx_imported_tables_begin(),
    //     vm_offsets.vmctx_imported_memories_begin(),
    //     vm_offsets.vmctx_imported_globals_begin(),
    //     vm_offsets.vmctx_tables_begin(),
    //     vm_offsets.vmctx_memories_begin(),
    //     vm_offsets.vmctx_globals_begin(),
    //     vm_offsets.vmctx_anyfuncs_begin(),
    //     vm_offsets.vmctx_builtin_functions_begin(),
    //     vm_offsets.size_of_vmctx(),
    //     );
    let mut offsets = HashMap::new();
    // 1. load signatures
    let sig_start = vm_offsets.vmctx_signature_ids_begin();
    let num_sigs = vm_offsets.num_signature_ids;
    // size = 4
    for offset in (sig_start..sig_start + num_sigs * 8).step_by(4) {
        offsets.insert(offset as i64, VmCtxField(FieldDesc::Rx));
    }
    // 2. load imported funcs
    let funcs_start = vm_offsets.vmctx_imported_functions_begin();
    let num_funcs = vm_offsets.num_imported_functions;
    for offset in (funcs_start..funcs_start + num_funcs * 16).step_by(8) {
        offsets.insert(offset as i64, VmCtxField(FieldDesc::Rx));
    }

    // 3. load tables
    let tables_start = vm_offsets.vmctx_tables_begin();
    let num_tables = vm_offsets.num_defined_tables;
    // table entries are 16 bytes: 8 bytes for base, 8 bytes for size
    for offset in (tables_start..tables_start + num_tables * 8 * 2).step_by(8 * 2) {
        offsets.insert(
            offset as i64,
            VmCtxField(FieldDesc::Ptr(Box::new(VmCtxField(FieldDesc::R)))),
        ); //base
        offsets.insert((offset + 8) as i64, VmCtxField(FieldDesc::R)); //size
    }

    // 4. load globals
    let globals_start = vm_offsets.vmctx_globals_begin();
    let num_globals = vm_offsets.num_defined_globals;
    // println!("Inserting globals {:?} {:?}", globals_start, num_globals);
    for offset in (globals_start..globals_start + num_globals * 8).step_by(8) {
        // println!("Inserting global = {:?}", offset);
        offsets.insert(offset as i64, VmCtxField(FieldDesc::Rw));
    }

    // 6. load HeapBase
    let mem_start = vm_offsets.vmctx_memories_begin();
    offsets.insert(mem_start as i64, HeapBase);

    // 7. load builtin functions
    let builtin_funcs_start = vm_offsets.vmctx_builtin_functions_begin();
    let builtin_funcs_end = vm_offsets.size_of_vmctx();
    for offset in (builtin_funcs_start..builtin_funcs_end).step_by(8) {
        offsets.insert(offset as i64, VmCtxField(FieldDesc::Rx));
    }
    //println!("Offsets = {:?}", offsets);
    // for (k,v) in offsets.iter(){
    //     println!("Offsets slot: {:?} {:?}", k, v);
    // }
    offsets
}

pub fn validate_wasmtime_func(
    code: &[u8],
    basic_blocks: &[usize],
    cfg_edges: &[(usize, usize)],
    arch_str: &str,
    func_name: String,
    vm_offsets: &wasmtime_environ::VMOffsets,
) -> Result<(), ValidationError> {
    // env_logger::init();
    // if func_name != "_wasm_function_569"{
    // if func_name != "u0:1435" {
    //     return Ok(());
    // }

    println!(
        "VeriWasm is verifying the Wasmtime aot compilation: {}",
        func_name
    );
    let arch = VwArch::from_str(arch_str).map_err(|err| ValidationError::Other(err))?;
    // println!("Arch = {:?}", arch);
    // println!("{:?} {:?} {:?}", code.len(), basic_blocks, cfg_edges);
    let cfg = get_cfg_from_compiler_info(code, basic_blocks, cfg_edges);
    let module = create_dummy_module(code, ExecutableType::Wasmtime, arch);
    match arch {
        VwArch::X64 => {
            let irmap = x64_lift_cfg(&module, &cfg, false);
        }
        VwArch::Aarch64 => {
            let irmap = aarch64_lift_cfg(&module, &cfg, false);
            // println!("CFG entry: {:x}", cfg.entrypoint);
            // for (_, block) in cfg.blocks.iter() {
            //     println!("CFG block: [0x{:x} : 0x{:x}]", block.start, block.end);
            // }
            // // println!("CFG graph: {:?}", cfg.graph);

            // for (baddr, block) in irmap.iter() {
            //     println!(
            //         "[{:x}:{:x}] ============",
            //         cfg.get_block(*baddr).start,
            //         cfg.get_block(*baddr).end
            //     );
            //     println!("Predecessors: {:x?}", cfg.predecessors(*baddr));
            //     println!("Destinations: {:x?}", cfg.destinations(*baddr));
            //     for (addr, ir_instr) in block.iter() {
            //         println!("  {:x} {:?}", addr, ir_instr);
            //     }
            // }
            //println!("{:?}", irmap);

            // let stack_analyzer = StackAnalyzer {};
            // let stack_result = run_worklist(&cfg, &irmap, &stack_analyzer);
            // let stack_safe = check_stack(stack_result, &irmap, &stack_analyzer);
            // if !stack_safe {
            //     return Err(ValidationError::StackUnsafe);
            // }
            println!("Checking heap for {}", func_name);
            let offsets = get_vm_offsets(vm_offsets);
            let wasmtime_analyzer = WasmtimeAnalyzer {
                offsets,
                name: func_name.clone(),
            };
            let wasmtime_result = run_worklist(&cfg, &irmap, &wasmtime_analyzer);
            let wasmtime_safe = check_wasmtime(wasmtime_result, &irmap, &wasmtime_analyzer);
            // println!("Wow, a bug!");
            // return Ok(());
            if !wasmtime_safe {
                println!("Veriwasm Check failed: {}", func_name);
                return Err(ValidationError::HeapUnsafe);
            }
        }
    }
    // let irmap: IRMap<X86Regs> = x64_lift_cfg(&module, &cfg, false);
    println!("Done!");
    Ok(())
}
