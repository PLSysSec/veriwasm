use crate::{analyses, checkers, ir, lattices, loaders};

use crate::lattices::VariableState;
use crate::VwModule;
use crate::{IRMap, VwMetadata, VW_CFG};
use analyses::{run_worklist, AnalysisResult};

use analyses::{HeapAnalyzer, StackAnalyzer};
use checkers::{check_heap, check_stack};
use ir::fully_resolved_cfg;
use ir::types::FunType;
use loaders::load_program;
use loaders::types::{ExecutableType, VwArch, VwFuncInfo};
use loaders::utils::get_data;
use loaders::utils::to_system_v;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter::FromIterator;

// use loaders::get_plt_funcs;
use loaders::Loadable;
use serde_json;
use std::fs;
use std::panic;
use std::time::Instant;
use yaxpeax_core::analyses::control_flow::check_cfg_integrity;

#[derive(Debug)]
pub struct PassConfig {
    pub stack: bool,
    pub linear_mem: bool,
    pub call: bool,
    pub zero_cost: bool,
}

pub struct Config {
    pub module_path: String,
    pub _num_jobs: u32,
    pub output_path: String,
    pub has_output: bool,
    pub only_func: Option<String>,
    pub executable_type: ExecutableType,
    pub active_passes: PassConfig,
    pub arch: VwArch,
    pub strict: bool,
}

fn run_stack(cfg: &VW_CFG, irmap: &IRMap) -> bool {
    let stack_analyzer = StackAnalyzer {};
    let stack_result = run_worklist(&cfg, &irmap, &stack_analyzer);
    let stack_safe = check_stack(stack_result, &irmap, &stack_analyzer);
    stack_safe
}

fn run_heap(
    cfg: &VW_CFG,
    irmap: &IRMap,
    metadata: &VwMetadata,
    all_addrs_map: &HashMap<u64, String>,
) -> bool {
    let heap_analyzer = HeapAnalyzer {
        metadata: metadata.clone(),
    };
    let heap_result = run_worklist(&cfg, &irmap, &heap_analyzer);
    let heap_safe = check_heap(heap_result, &irmap, &heap_analyzer, &all_addrs_map);
    heap_safe
}

pub fn run(config: Config) {
    let module = load_program(&config);
    // We only need to load the data if we are doing zero cost checks
    if config.active_passes.zero_cost {
        let plt_funcs = config
            .executable_type
            .get_plt_funcs(&config.module_path)
            .unwrap_or(Vec::new());
        let func_signatures = config.executable_type.get_func_signatures(&module.program);
        return run_helper(config, module, plt_funcs, func_signatures);
    }
    //let plt_funcs = config.executable_type.get_plt_funcs(&config.module_path);
    let plt_funcs = Vec::new();
    // all_addrs.extend(plt_funcs);
    //let func_signatures = config.executable_type.get_func_signatures(&module.program);
    let func_signatures = VwFuncInfo::new();
    run_helper(config, module, plt_funcs, func_signatures);
}

pub fn run_helper(
    config: Config,
    module: VwModule,
    plt_funcs: Vec<(u64, String)>,
    func_signatures: VwFuncInfo,
) {
    // let module = load_program(&config);

    // let plt_funcs = config.executable_type.get_plt_funcs(&config.module_path);
    // all_addrs.extend(plt_funcs);
    // let func_signatures = config.executable_type.get_func_signatures(&module.program);

    let (x86_64_data, func_addrs, plt, mut all_addrs) =
        get_data(&module.program, &config.executable_type);
    all_addrs.extend(plt_funcs);

    let strict = config.strict;

    let mut func_counter = 0;
    let mut info: Vec<(std::string::String, usize, f64, f64, f64)> = vec![];
    let valid_funcs: Vec<u64> = func_addrs.clone().iter().map(|x| x.0).collect();
    let all_addrs_map = HashMap::from_iter(all_addrs.clone());
    for (addr, func_name) in func_addrs {
        if config.only_func.is_some() && func_name != config.only_func.as_ref().unwrap().as_str() {
            continue;
        }
        println!("Generating CFG for {:?}", func_name);
        let start = Instant::now();
        let (cfg, irmap) = fully_resolved_cfg(&module, &x86_64_data.contexts, addr, strict);
        func_counter += 1;
        println!("Analyzing 0x{:x?}: {:?}", addr, func_name);
        check_cfg_integrity(&cfg.blocks, &cfg.graph);

        let stack_start = Instant::now();
        if config.active_passes.stack {
            println!("Checking Stack Safety");
            let stack_safe = run_stack(&cfg, &irmap);
            if !stack_safe {
                panic!("Not Stack Safe");
            }
        }

        let heap_start = Instant::now();
        if config.active_passes.linear_mem {
            println!("Checking Heap Safety");
            let heap_safe = run_heap(&cfg, &irmap, &module.metadata, &all_addrs_map);
            if !heap_safe {
                panic!("Not Heap Safe");
            }
        }

        let end = Instant::now();
        info.push((
            func_name.to_string(),
            cfg.blocks.len(),
            (stack_start - start).as_secs_f64(),
            (heap_start - stack_start).as_secs_f64(),
            (end - heap_start).as_secs_f64(),
        ));
        println!(
            "Verified {:?} at {:?} blocks. CFG: {:?}s Stack: {:?}s Heap: {:?}s",
            func_name,
            cfg.blocks.len(),
            (stack_start - start).as_secs_f64(),
            (heap_start - stack_start).as_secs_f64(),
            (end - heap_start).as_secs_f64(),
        );
    }
    if config.has_output {
        let data = serde_json::to_string(&info).unwrap();
        println!("Dumping Stats to {}", config.output_path);
        fs::write(config.output_path, data).expect("Unable to write file");
    }

    let mut total_cfg_time = 0.0;
    let mut total_stack_time = 0.0;
    let mut total_heap_time = 0.0;
    for (_, _, cfg_time, stack_time, heap_time) in &info {
        total_cfg_time += cfg_time;
        total_stack_time += stack_time;
        total_heap_time += heap_time;
    }
    println!("Verified {:?} functions", func_counter);
    println!(
        "Total time = {:?}s CFG: {:?} Stack: {:?}s Heap: {:?}s",
        total_cfg_time + total_stack_time + total_heap_time,
        total_cfg_time,
        total_stack_time,
        total_heap_time,
    );
    println!("Done!");
}
