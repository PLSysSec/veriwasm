pub mod lattices;
pub mod utils;
pub mod analyses;
pub mod metadata;
pub mod lifter;
pub mod ir_utils;
pub mod checkers;
pub mod cfg;
use crate::analyses::reaching_defs::ReachingDefnAnalyzer;
use crate::analyses::jump_analyzer::SwitchAnalyzer;
use crate::checkers::jump_resolver::resolve_jumps;
use crate::checkers::call_checker::check_calls;
use crate::analyses::call_analyzer::CallAnalyzer;
use crate::analyses::heap_analyzer::HeapAnalyzer;
use crate::analyses::run_worklist;
use crate::analyses::stack_analyzer::StackAnalyzer;
use crate::lifter::IRMap;
use crate::analyses::jump_analyzer::analyze_jumps;
use crate::analyses::reaching_defs::analyze_reaching_defs;
use utils::{load_program, get_cfgs, load_metadata};
use clap::{Arg, App};
use lifter::{lift_cfg, Stmt, Value};
use crate::checkers::stack_checker::check_stack;
use crate::checkers::heap_checker::check_heap;

pub struct Config{
    module_path: String,
    num_jobs: u32,
    output_path : String,
    has_output : bool,
    quiet : bool
}

fn has_indirect_calls(irmap: &IRMap) -> bool{
    for (block_addr, ir_block) in irmap {
        for (addr,ir_stmts) in ir_block{
            for (idx,ir_stmt) in ir_stmts.iter().enumerate(){
                match ir_stmt{
                    Stmt::Call(Value::Reg(_,_)) | Stmt::Call(Value::Mem(_,_)) => return true,
                    _ => ()
                }
            }
        }
    }
    false
}

fn has_indirect_jumps(irmap: &IRMap) -> bool{
    for (block_addr, ir_block) in irmap {
        for (addr,ir_stmts) in ir_block{
            for (idx,ir_stmt) in ir_stmts.iter().enumerate(){
                match ir_stmt{
                    Stmt::Branch(_,Value::Reg(_,_)) | Stmt::Branch(_,Value::Mem(_,_)) => return true,
                    _ => ()
                }
            }
        }
    }
    false
}

fn run(config : Config){
    let program = load_program(&config.module_path);
    // let cfgs = get_cfgs(binpath);
    println!("Loading Metadata");
    let metadata = load_metadata(&config.module_path);
    for (func_name,cfg) in get_cfgs(&config.module_path).iter(){
    // for (func_name,cfg) in get_resolved_cfgs(&config.module_path).iter(){

        println!("Analyzing: {:?}", func_name);
        let irmap = lift_cfg(&program, &cfg, &metadata);
        let reaching_defs = analyze_reaching_defs(cfg, &irmap, metadata.clone());
       
        let stack_analyzer = StackAnalyzer{};
        let stack_result = run_worklist(cfg, &irmap, &stack_analyzer); 
        let stack_safe = check_stack(stack_result, &irmap, &stack_analyzer);
        assert!(stack_safe);
        println!("Checking Heap Safety");
        let heap_analyzer = HeapAnalyzer{metadata : metadata.clone()};
        let heap_result = run_worklist(cfg, &irmap, &heap_analyzer); 
        let heap_safe = check_heap(heap_result, &irmap, &heap_analyzer);
        assert!(heap_safe);
        println!("Checking Call Safety");
        if has_indirect_calls(&irmap){
            let call_analyzer = CallAnalyzer{metadata : metadata.clone(), reaching_defs : reaching_defs.clone(), reaching_analyzer : ReachingDefnAnalyzer{}};
            let call_result = run_worklist(cfg, &irmap, &call_analyzer);    
            let call_safe = check_calls(call_result, &irmap, &call_analyzer);
            assert!(call_safe);
        }
    }
    println!("Done!");
}

fn main() {
    let matches = App::new("VeriWasm")
    .version("0.1.0")
    .about("Validates safety of native Wasm module")
    .arg(Arg::with_name("module path")
        .short("i")
        .takes_value(true)
        .help("path to native Wasm module to validate")
        .required(true))
    .arg(Arg::with_name("jobs")
        .short("j")
        .long("jobs")
        .takes_value(true)
        .help("Number of parallel threads (default 1)"))
    .arg(Arg::with_name("stats output path")
        .short("o")
        .long("output")
        .takes_value(true)
        .help("Path to output stats file"))
    .arg(Arg::with_name("quiet")
        .short("q")
        .long("quiet"))
    .get_matches();

    let module_path = matches.value_of("module path").unwrap();
    let num_jobs_opt = matches.value_of("jobs");
    let output_path = matches.value_of("stats output path").unwrap_or("");
    let num_jobs  = num_jobs_opt.map(|s| s.parse::<u32>().unwrap_or(1)).unwrap_or(1);
    let quiet = matches.is_present("quiet");

    let has_output = if output_path == "" {true} else {false};

    let metadata_path = module_path.clone();

    let config = Config{
        module_path: module_path.to_string(), 
        num_jobs: num_jobs, 
        output_path : output_path.to_string(),
        has_output : has_output, 
        quiet:quiet};

    run(config);
    // println!("wow metadata_path = {:?}", metadata_path);
    // load_metadata(metadata_path.to_string());
}

//TODO: get all tests to pass

fn lift_test_helper(path: &str){
    let program = load_program(path);
    let metadata = load_metadata(path);
    for (func_name,cfg) in get_cfgs(&path).iter(){
        println!("{}",func_name);
        let irmap = lift_cfg(&program, cfg, &metadata);
        let reaching_defs = analyze_reaching_defs(cfg, &irmap, metadata.clone());
        
        let switch_analyzer = SwitchAnalyzer{metadata : metadata.clone(), reaching_defs : reaching_defs.clone(), reaching_analyzer : ReachingDefnAnalyzer{}};
        let switch_results = analyze_jumps(cfg, &irmap, &switch_analyzer);
        let switch_targets = resolve_jumps(&program, switch_results, &irmap, &switch_analyzer);
                
        let stack_analyzer = StackAnalyzer{};
        let stack_result = run_worklist(cfg, &irmap, &stack_analyzer); 
        let stack_safe = check_stack(stack_result, &irmap, &StackAnalyzer{});
        assert!(stack_safe);

        let heap_analyzer = HeapAnalyzer{metadata : metadata.clone()};
        let heap_result = run_worklist(cfg, &irmap, &heap_analyzer); 
        let heap_safe = check_heap(heap_result, &irmap, &HeapAnalyzer{metadata : metadata.clone()});
        // assert!(heap_safe);

        let call_analyzer = CallAnalyzer{metadata : metadata.clone(), reaching_defs : reaching_defs.clone(),reaching_analyzer : ReachingDefnAnalyzer{}};
        let call_result = run_worklist(cfg, &irmap, &call_analyzer);    
        let call_safe = check_calls(call_result, &irmap, &call_analyzer);
        // assert!(call_safe);

        //&CallAnalyzer{metadata : metadata.clone(), reaching_defs :
        //reaching_defs});    
    }
}

#[test]
fn full_test_unit_tests() {
    lift_test_helper("./veriwasm_data/stack_check_unit_tests.so")
}

#[test]
fn full_test_libgraphite() {
    lift_test_helper("./veriwasm_data/firefox_libs/libgraphitewasm.so")
}

#[test]
fn full_test_libogg() {
    lift_test_helper("./veriwasm_data/firefox_libs/liboggwasm.so")
}

#[test]
fn full_test_shootout() {
    lift_test_helper("./veriwasm_data/shootout/shootout.so")
}


#[test]
fn full_test_astar() {
    lift_test_helper("./veriwasm_data/spec/astar_base.wasm_lucet")
}

#[test]
fn full_test_gobmk() {
    lift_test_helper("./veriwasm_data/spec/gobmk_base.wasm_lucet")
}

#[test]
fn full_test_lbm() {
    lift_test_helper("./veriwasm_data/spec/lbm_base.wasm_lucet")
}


#[test]
fn full_test_mcf() {
    lift_test_helper("./veriwasm_data/spec/mcf_base.wasm_lucet")
}

#[test]
fn full_test_namd() {
    lift_test_helper("./veriwasm_data/spec/namd_base.wasm_lucet")
}

#[test]
fn full_test_sjeng() {
    lift_test_helper("./veriwasm_data/spec/sjeng_base.wasm_lucet")
}


#[test]
fn full_test_sphinx_livepretend() {
    lift_test_helper("./veriwasm_data/spec/sphinx_livepretend_base.wasm_lucet")
}

#[test]
fn full_test_bzip2() {
    lift_test_helper("./veriwasm_data/spec/bzip2_base.wasm_lucet")
}

#[test]
fn full_test_h264ref() {
    lift_test_helper("./veriwasm_data/spec/h264ref_base.wasm_lucet")
}

#[test]
fn full_test_libquantum() {
    lift_test_helper("./veriwasm_data/spec/libquantum_base.wasm_lucet")
}

#[test]
fn full_test_milc() {
    lift_test_helper("./veriwasm_data/spec/milc_base.wasm_lucet")
}

#[test]
fn full_test_povray() {
    lift_test_helper("./veriwasm_data/spec/povray_base.wasm_lucet")
}

#[test]
fn full_test_soplex() {
    lift_test_helper("./veriwasm_data/spec/soplex_base.wasm_lucet")
}

