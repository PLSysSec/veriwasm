use veriwasm::{analyses, checkers, loaders, utils};

use analyses::call_analyzer::CallAnalyzer;
use analyses::heap_analyzer::HeapAnalyzer;
use analyses::reaching_defs::{analyze_reaching_defs, ReachingDefnAnalyzer};
use analyses::run_worklist;
use analyses::stack_analyzer::StackAnalyzer;
use checkers::call_checker::check_calls;
use checkers::heap_checker::check_heap;
use checkers::stack_checker::check_stack;
use loaders::ExecutableType;
use utils::ir_utils::has_indirect_calls;
use utils::utils::{fully_resolved_cfg, get_data};

use clap::{App, Arg};
use serde_json;
use std::fs;
use std::panic;
use std::str::FromStr;
use std::time::Instant;
use utils::utils::load_metadata;
use veriwasm::loaders::Loadable;
use yaxpeax_core::analyses::control_flow::check_cfg_integrity;

pub struct Config {
    module_path: String,
    _num_jobs: u32,
    output_path: String,
    has_output: bool,
    _quiet: bool,
    only_func: Option<String>,
    executable_type: ExecutableType,
}

fn run(config: Config) {
    // if let ExecutableType::Wasmtime = config.executable_type{
    //     println!("This is wasmtime!");
    //     return;
    // }

    let program = config.executable_type.load_program(&config.module_path);
    let metadata = load_metadata(&program);
    let (x86_64_data, func_addrs, plt) = get_data(&program);

    let mut func_counter = 0;
    let mut info: Vec<(std::string::String, usize, f64, f64, f64, f64)> = vec![];
    let valid_funcs: Vec<u64> = func_addrs.clone().iter().map(|x| x.0).collect();
    for (addr, func_name) in func_addrs {
        if config.only_func.is_some() && func_name != config.only_func.as_ref().unwrap().as_str() {
            continue;
        }
        println!("Generating CFG for {:?}", func_name);
        let start = Instant::now();
        let (cfg, irmap) = fully_resolved_cfg(&program, &x86_64_data.contexts, &metadata, addr);
        func_counter += 1;
        println!("Analyzing: {:?}", func_name);
        //let irmap = lift_cfg(&program, &cfg, &metadata);
        check_cfg_integrity(&cfg.blocks, &cfg.graph);

        let stack_start = Instant::now();
        let stack_analyzer = StackAnalyzer {};
        let stack_result = run_worklist(&cfg, &irmap, &stack_analyzer);
        let stack_safe = check_stack(stack_result, &irmap, &stack_analyzer);
        if !stack_safe {
            panic!("Not Stack Safe");
        }

        println!("Checking Heap Safety");
        let heap_start = Instant::now();
        let heap_analyzer = HeapAnalyzer {
            metadata: metadata.clone(),
        };
        let heap_result = run_worklist(&cfg, &irmap, &heap_analyzer);
        let heap_safe = check_heap(heap_result, &irmap, &heap_analyzer);
        if !heap_safe {
            panic!("Not Heap Safe");
        }

        let call_start = Instant::now();
        println!("Checking Call Safety");
        if has_indirect_calls(&irmap) {
            let reaching_defs = analyze_reaching_defs(&cfg, &irmap, metadata.clone());
            let call_analyzer = CallAnalyzer {
                metadata: metadata.clone(),
                reaching_defs: reaching_defs.clone(),
                reaching_analyzer: ReachingDefnAnalyzer {
                    cfg: cfg.clone(),
                    irmap: irmap.clone(),
                },
                funcs: valid_funcs.clone(),
            };
            let call_result = run_worklist(&cfg, &irmap, &call_analyzer);
            let call_safe = check_calls(call_result, &irmap, &call_analyzer, &valid_funcs, &plt);
            if !call_safe {
                panic!("Not Call Safe");
            }
        }
        let end = Instant::now();
        info.push((
            func_name.to_string(),
            cfg.blocks.len(),
            (stack_start - start).as_secs_f64(),
            (heap_start - stack_start).as_secs_f64(),
            (call_start - heap_start).as_secs_f64(),
            (end - call_start).as_secs_f64(),
        ));
        println!(
            "Verified {:?} at {:?} blocks. CFG: {:?}s Stack: {:?}s Heap: {:?}s Calls: {:?}s",
            func_name,
            cfg.blocks.len(),
            (stack_start - start).as_secs_f64(),
            (heap_start - stack_start).as_secs_f64(),
            (call_start - heap_start).as_secs_f64(),
            (end - call_start).as_secs_f64()
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
    let mut total_call_time = 0.0;
    for (_, _, cfg_time, stack_time, heap_time, call_time) in &info {
        total_cfg_time += cfg_time;
        total_stack_time += stack_time;
        total_heap_time += heap_time;
        total_call_time += call_time;
    }
    println!("Verified {:?} functions", func_counter);
    println!(
        "Total time = {:?}s CFG: {:?} Stack: {:?}s Heap: {:?}s Call: {:?}s",
        total_cfg_time + total_stack_time + total_heap_time + total_call_time,
        total_cfg_time,
        total_stack_time,
        total_heap_time,
        total_call_time
    );
    println!("Done!");
}

fn main() {
    let _ = env_logger::try_init();
    let matches = App::new("VeriWasm")
        .version("0.1.0")
        .about("Validates safety of native Wasm code")
        .arg(
            Arg::with_name("module path")
                .short("i")
                .takes_value(true)
                .help("path to native Wasm module to validate")
                .required(true),
        )
        .arg(
            Arg::with_name("jobs")
                .short("j")
                .long("jobs")
                .takes_value(true)
                .help("Number of parallel threads (default 1)"),
        )
        .arg(
            Arg::with_name("stats output path")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("Path to output stats file"),
        )
        .arg(
            Arg::with_name("one function")
                .short("f")
                .long("func")
                .takes_value(true)
                .help("Single function to process (rather than whole module)"),
        )
        .arg(
            Arg::with_name("executable type")
                .short("c")
                .long("format")
                .takes_value(true)
                .help("Format of the executable (lucet | wasmtime)"),
        )
        .arg(Arg::with_name("quiet").short("q").long("quiet"))
        .get_matches();

    let module_path = matches.value_of("module path").unwrap();
    let num_jobs_opt = matches.value_of("jobs");
    let output_path = matches.value_of("stats output path").unwrap_or("");
    let num_jobs = num_jobs_opt
        .map(|s| s.parse::<u32>().unwrap_or(1))
        .unwrap_or(1);
    let quiet = matches.is_present("quiet");
    let only_func = matches.value_of("one function").map(|s| s.to_owned());
    let executable_type =
        ExecutableType::from_str(matches.value_of("executable type").unwrap_or("lucet")).unwrap();

    let has_output = if output_path == "" { false } else { true };

    let config = Config {
        module_path: module_path.to_string(),
        _num_jobs: num_jobs,
        output_path: output_path.to_string(),
        has_output: has_output,
        _quiet: quiet,
        only_func,
        executable_type,
    };

    run(config);
}
