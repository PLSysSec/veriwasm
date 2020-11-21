pub mod lattices;
pub mod utils;
pub mod analyses;
pub mod metadata;
pub mod lifter;
pub mod ir_utils;
pub mod checkers;
pub mod cfg;
use crate::utils::get_one_resolved_cfg;
use yaxpeax_core::analyses::control_flow::check_cfg_integrity;
use crate::utils::get_resolved_cfgs;
use crate::analyses::reaching_defs::ReachingDefnAnalyzer;
use crate::checkers::call_checker::check_calls;
use crate::analyses::call_analyzer::CallAnalyzer;
use crate::analyses::heap_analyzer::HeapAnalyzer;
use crate::analyses::run_worklist;
use crate::analyses::stack_analyzer::StackAnalyzer;
use crate::lifter::IRMap;
use crate::analyses::reaching_defs::analyze_reaching_defs;
use utils::{load_program, load_metadata};
use clap::{Arg, App};
use lifter::{Stmt, Value};
use crate::checkers::stack_checker::check_stack;
use crate::checkers::heap_checker::check_heap;
use std::panic;

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
    println!("Loading Metadata");
    let metadata = load_metadata(&config.module_path);
    // for (func_name,cfg) in get_cfgs(&config.module_path).iter(){
    for (func_name,(cfg,irmap)) in get_resolved_cfgs(&config.module_path).iter(){
        println!("Analyzing: {:?}", func_name);
        //let irmap = lift_cfg(&program, &cfg, &metadata);
        check_cfg_integrity(&cfg.blocks,&cfg.graph);
        // assert_eq!(cfg.blocks.keys(), ir);
       
        let stack_analyzer = StackAnalyzer{};
        let stack_result = run_worklist(&cfg, &irmap, &stack_analyzer); 
        let stack_safe = check_stack(stack_result, &irmap, &stack_analyzer);
        assert!(stack_safe);
        println!("Checking Heap Safety");
        let heap_analyzer = HeapAnalyzer{metadata : metadata.clone()};
        let heap_result = run_worklist(&cfg, &irmap, &heap_analyzer); 
        let heap_safe = check_heap(heap_result, &irmap, &heap_analyzer);
        assert!(heap_safe);
        println!("Checking Call Safety");
        if has_indirect_calls(&irmap){
            let reaching_defs = analyze_reaching_defs(&cfg, &irmap, metadata.clone());
            let call_analyzer = CallAnalyzer{metadata : metadata.clone(), reaching_defs : reaching_defs.clone(), reaching_analyzer : ReachingDefnAnalyzer{}};
            let call_result = run_worklist(&cfg, &irmap, &call_analyzer);    
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

fn full_test_helper(path: &str){
    let program = load_program(&path);
    println!("Loading Metadata");
    let metadata = load_metadata(&path);
    for (func_name,(cfg,irmap)) in get_resolved_cfgs(&path).iter(){
        println!("Analyzing: {:?}", func_name);
        check_cfg_integrity(&cfg.blocks,&cfg.graph);       
        let stack_analyzer = StackAnalyzer{};
        let stack_result = run_worklist(&cfg, &irmap, &stack_analyzer); 
        let stack_safe = check_stack(stack_result, &irmap, &stack_analyzer);
        assert!(stack_safe);
        println!("Checking Heap Safety");
        let heap_analyzer = HeapAnalyzer{metadata : metadata.clone()};
        let heap_result = run_worklist(&cfg, &irmap, &heap_analyzer); 
        let heap_safe = check_heap(heap_result, &irmap, &heap_analyzer);
        assert!(heap_safe);
        println!("Checking Call Safety");
        if has_indirect_calls(&irmap){
            let reaching_defs = analyze_reaching_defs(&cfg, &irmap, metadata.clone());
            let call_analyzer = CallAnalyzer{metadata : metadata.clone(), reaching_defs : reaching_defs.clone(), reaching_analyzer : ReachingDefnAnalyzer{}};
            let call_result = run_worklist(&cfg, &irmap, &call_analyzer);    
            let call_safe = check_calls(call_result, &irmap, &call_analyzer);
            assert!(call_safe);
        }
    }
    println!("Done!");
}


fn negative_test_helper(path: &str, func_name: &str){
    let program = load_program(&path);
    println!("Loading Metadata");
    let metadata = load_metadata(&path);
    let (cfg,irmap) = get_one_resolved_cfg(path,func_name);
    println!("Analyzing: {:?}", func_name);
    check_cfg_integrity(&cfg.blocks,&cfg.graph);       
    let stack_analyzer = StackAnalyzer{};
    let stack_result = run_worklist(&cfg, &irmap, &stack_analyzer); 
    let stack_safe = check_stack(stack_result, &irmap, &stack_analyzer);
    assert!(stack_safe);
    println!("Checking Heap Safety");
    let heap_analyzer = HeapAnalyzer{metadata : metadata.clone()};
    let heap_result = run_worklist(&cfg, &irmap, &heap_analyzer); 
    let heap_safe = check_heap(heap_result, &irmap, &heap_analyzer);
    assert!(heap_safe);
    println!("Checking Call Safety");
    if has_indirect_calls(&irmap){
        let reaching_defs = analyze_reaching_defs(&cfg, &irmap, metadata.clone());
        let call_analyzer = CallAnalyzer{metadata : metadata.clone(), reaching_defs : reaching_defs.clone(), reaching_analyzer : ReachingDefnAnalyzer{}};
        let call_result = run_worklist(&cfg, &irmap, &call_analyzer);    
        let call_safe = check_calls(call_result, &irmap, &call_analyzer);
        assert!(call_safe);
        
    }
    println!("Done!");
}


#[test]
fn full_test_unit_tests() {
    full_test_helper("./veriwasm_data/stack_check_unit_tests.so")
}

#[test]
fn full_test_libgraphite() {
    full_test_helper("./veriwasm_data/firefox_libs/libgraphitewasm.so")
}

#[test]
fn full_test_libogg() {
    full_test_helper("./veriwasm_data/firefox_libs/liboggwasm.so")
}

#[test]
fn full_test_shootout() {
    full_test_helper("./veriwasm_data/shootout/shootout.so")
}


#[test]
fn full_test_astar() {
    full_test_helper("./veriwasm_data/spec/astar_base.wasm_lucet")
}

#[test]
fn full_test_gobmk() {
    full_test_helper("./veriwasm_data/spec/gobmk_base.wasm_lucet")
}

#[test]
fn full_test_lbm() {
    full_test_helper("./veriwasm_data/spec/lbm_base.wasm_lucet")
}


#[test]
fn full_test_mcf() {
    full_test_helper("./veriwasm_data/spec/mcf_base.wasm_lucet")
}

#[test]
fn full_test_namd() {
    full_test_helper("./veriwasm_data/spec/namd_base.wasm_lucet")
}

#[test]
fn full_test_sjeng() {
    full_test_helper("./veriwasm_data/spec/sjeng_base.wasm_lucet")
}


#[test]
fn full_test_sphinx_livepretend() {
    full_test_helper("./veriwasm_data/spec/sphinx_livepretend_base.wasm_lucet")
}

#[test]
fn full_test_bzip2() {
    full_test_helper("./veriwasm_data/spec/bzip2_base.wasm_lucet")
}

#[test]
fn full_test_h264ref() {
    full_test_helper("./veriwasm_data/spec/h264ref_base.wasm_lucet")
}

#[test]
fn full_test_libquantum() {
    full_test_helper("./veriwasm_data/spec/libquantum_base.wasm_lucet")
}

#[test]
fn full_test_milc() {
    full_test_helper("./veriwasm_data/spec/milc_base.wasm_lucet")
}

#[test]
fn full_test_povray() {
    full_test_helper("./veriwasm_data/spec/povray_base.wasm_lucet")
}

#[test]
fn full_test_soplex() {
    full_test_helper("./veriwasm_data/spec/soplex_base.wasm_lucet")
}


#[test]
#[should_panic]
fn negative_test_1() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_1_testfail");
}

#[test]
#[should_panic]
fn negative_test_2() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_2_testfail");
}

#[test]
#[should_panic]
fn negative_test_3() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_3_testfail");
}

#[test]
#[should_panic]
fn negative_test_4() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_4_testfail");
}

#[test]
#[should_panic]
fn negative_test_5() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_5_testfail");
}

#[test]
#[should_panic]
fn negative_test_6() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_6_testfail");
}

#[test]
#[should_panic]
fn negative_test_7() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_7_testfail");
}

#[test]
#[should_panic]
fn negative_test_8() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_8_testfail");
}

#[test]
#[should_panic]
fn negative_test_9() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_9_testfail");
}

#[test]
#[should_panic]
fn negative_test_10() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_10_testfail");
}

#[test]
#[should_panic]
fn negative_test_11() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_11_testfail");
}

#[test]
#[should_panic]
fn negative_test_12() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_12_testfail");
}

#[test]
#[should_panic]
fn negative_test_13() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_13_testfail");
}

#[test]
#[should_panic]
fn negative_test_14() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_14_testfail");
}

// # NaCl issue #23
#[test]
#[should_panic]
fn negative_test_nacl_23() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_nacl_23");
}

#[test]
#[should_panic]
fn negative_test_nacl_323_1() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_nacl_323_1");
}

#[test]
#[should_panic]
fn negative_test_nacl_323_2() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_nacl_323_2");
}

#[test]
#[should_panic]
fn negative_nacl_323_3() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_nacl_323_3");
}

#[test]
#[should_panic]
fn negative_nacl_323_4() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_nacl_323_4");
}

#[test]
#[should_panic]
fn negative_test_nacl_390() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_nacl_390");
}

#[test]
#[should_panic]
fn negative_test_nacl_1585() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_nacl_1585");
}

#[test]
#[should_panic]
fn negative_test_nacl_2532() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_nacl_2532");
}

#[test]
#[should_panic]
fn negative_test_bakersfield_1() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_bakersfield_1");
}

#[test]
#[should_panic]
fn negative_test_misfit_1() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_misfit_1");
}

#[test]
#[should_panic]
fn negative_test_cranelift_805() {
    negative_test_helper("veriwasm_data/negative_tests/negative_tests.so", "guest_func_cranelift_805");
}
