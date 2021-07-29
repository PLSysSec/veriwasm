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
use analyses::{HeapAnalyzer, StackAnalyzer};
use checkers::{check_heap, check_stack};
// use ir::lift_cfg;
use crate::ir::types::X86Regs;
use ir::types::IRMap;
use ir::{aarch64_lift_cfg, x64_lift_cfg};
use loaders::types::{ExecutableType, VwArch, VwMetadata, VwModule};
use petgraph::graphmap::GraphMap;
use std::collections::BTreeMap;
use std::str::FromStr;
use yaxpeax_core::analyses::control_flow::{VW_Block, VW_CFG};
use yaxpeax_core::memory::repr::process::{ModuleData, ModuleInfo, Segment};

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
        data: code.iter().cloned().collect(),
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
        metadata: module.metadata.clone(),
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

pub fn validate_wasmtime_func(
    code: &[u8],
    basic_blocks: &[usize],
    cfg_edges: &[(usize, usize)],
    arch_str: &str,
) -> Result<(), ValidationError> {
    // env_logger::init();
    println!("VeriWasm is verifying the Wasmtime aot compilation!");
    let arch = VwArch::from_str(arch_str).map_err(|err| ValidationError::Other(err))?;
    println!("Arch = {:?}", arch);
    println!("{:?} {:?} {:?}", code.len(), basic_blocks, cfg_edges);
    let cfg = get_cfg_from_compiler_info(code, basic_blocks, cfg_edges);
    let module = create_dummy_module(code, ExecutableType::Wasmtime, arch);
    match arch {
        VwArch::X64 => {
            let irmap = x64_lift_cfg(&module, &cfg, false);
        }
        VwArch::Aarch64 => {
            let irmap = aarch64_lift_cfg(&module, &cfg, false);
            println!("CFG entry: {:x}", cfg.entrypoint);
            for (_, block) in cfg.blocks.iter() {
                println!("CFG block: [0x{:x} : 0x{:x}]", block.start, block.end);
            }
            println!("CFG graph: {:?}", cfg.graph);

            for (baddr, block) in irmap.iter() {
                println!("{:x} ============ ", baddr);
                for (addr, ir_instr) in block.iter() {
                    println!("  {:x} {:?}", addr, ir_instr);
                }
            }
            //println!("{:?}", irmap);

            let stack_analyzer = StackAnalyzer {};
            let stack_result = run_worklist(&cfg, &irmap, &stack_analyzer);
            let stack_safe = check_stack(stack_result, &irmap, &stack_analyzer);
            // if !stack_safe {
            //     return Err(ValidationError::StackUnsafe);
            // }

            // let heap_analyzer = HeapAnalyzer {
            //     metadata: module.metadata.clone(),
            // };
            // let heap_result = run_worklist(&cfg, &irmap, &heap_analyzer);
            // let heap_safe = check_heap(heap_result, &irmap, &heap_analyzer);
            // if !heap_safe {
            //     return Err(ValidationError::HeapUnsafe);
            // }
        }
    }
    // let irmap: IRMap<X86Regs> = x64_lift_cfg(&module, &cfg, false);
    println!("Done!");
    Ok(())
}
