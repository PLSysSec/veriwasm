use crate::analyses::call_analyzer::CallAnalyzer;
use crate::analyses::heap_analyzer::HeapAnalyzer;
use crate::analyses::reaching_defs::{analyze_reaching_defs, ReachingDefnAnalyzer};
use crate::analyses::run_worklist;
use crate::analyses::stack_analyzer::StackAnalyzer;
use crate::checkers::call_checker::check_calls;
use crate::checkers::heap_checker::check_heap;
use crate::checkers::stack_checker::check_stack;
use crate::utils::ir_utils::has_indirect_calls;
use crate::utils::utils::{fully_resolved_cfg, get_data, get_one_resolved_cfg};
use crate::utils::utils::{load_metadata, load_program};
use std::panic;
use yaxpeax_core::analyses::control_flow::check_cfg_integrity;

fn full_test_helper(path: &str) {
    let program = load_program(&path);
    println!("Loading Metadata");
    let metadata = load_metadata(&path);
    let (x86_64_data, func_addrs, plt) = get_data(&path, &program);
    let valid_funcs: Vec<u64> = func_addrs.clone().iter().map(|x| x.0).collect();
    for (addr, _func_name) in func_addrs {
        let (cfg, irmap) = fully_resolved_cfg(&program, &x86_64_data.contexts, &metadata, addr);
        check_cfg_integrity(&cfg.blocks, &cfg.graph);
        let stack_analyzer = StackAnalyzer {};
        let stack_result = run_worklist(&cfg, &irmap, &stack_analyzer);
        let stack_safe = check_stack(stack_result, &irmap, &stack_analyzer);
        assert!(stack_safe);
        println!("Checking Heap Safety");
        let heap_analyzer = HeapAnalyzer {
            metadata: metadata.clone(),
        };
        let heap_result = run_worklist(&cfg, &irmap, &heap_analyzer);
        let heap_safe = check_heap(heap_result, &irmap, &heap_analyzer);
        assert!(heap_safe);
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
                funcs: vec![],
            };
            let call_result = run_worklist(&cfg, &irmap, &call_analyzer);
            let call_safe = check_calls(call_result, &irmap, &call_analyzer, &valid_funcs, &plt);
            assert!(call_safe);
        }
    }
    println!("Done!");
}

fn negative_test_helper(path: &str, func_name: &str) {
    let program = load_program(&path);
    let (x86_64_data, func_addrs, plt) = get_data(&path, &program);
    let valid_funcs: Vec<u64> = func_addrs.clone().iter().map(|x| x.0).collect();
    println!("Loading Metadata");
    let metadata = load_metadata(&path);
    let ((cfg, irmap), x86_64_data) = get_one_resolved_cfg(path, func_name);
    println!("Analyzing: {:?}", func_name);
    check_cfg_integrity(&cfg.blocks, &cfg.graph);
    println!("Checking Stack Safety");
    let stack_analyzer = StackAnalyzer {};
    let stack_result = run_worklist(&cfg, &irmap, &stack_analyzer);
    let stack_safe = check_stack(stack_result, &irmap, &stack_analyzer);
    assert!(stack_safe);
    println!("Checking Heap Safety");
    let heap_analyzer = HeapAnalyzer {
        metadata: metadata.clone(),
    };
    let heap_result = run_worklist(&cfg, &irmap, &heap_analyzer);
    let heap_safe = check_heap(heap_result, &irmap, &heap_analyzer);
    assert!(heap_safe);
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
            funcs: vec![],
        };
        let call_result = run_worklist(&cfg, &irmap, &call_analyzer);
        let call_safe = check_calls(call_result, &irmap, &call_analyzer, &valid_funcs, &plt);
        assert!(call_safe);
    }
    println!("Done! ");
}

#[test]
fn full_test_libgraphite() {
    full_test_helper("./veriwasm_public_data/firefox_libs/libgraphitewasm.so")
}

#[test]
fn full_test_libogg() {
    full_test_helper("./veriwasm_public_data/firefox_libs/liboggwasm.so")
}

#[test]
#[should_panic(expected = "assertion failed: stack_safe")]
fn negative_test_1() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_1_testfail",
    );
}

#[test]
#[should_panic(expected = "assertion failed: stack_safe")]
fn negative_test_2() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_2_testfail",
    );
}

#[test]
#[should_panic(expected = "assertion failed: stack_safe")]
fn negative_test_3() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_3_testfail",
    );
}

#[test]
#[should_panic(expected = "Jump Targets Broken, target = None")]
fn negative_test_4() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_4_testfail",
    );
}

#[test]
#[should_panic(expected = "assertion failed: call_safe")]
fn negative_test_5() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_5_testfail",
    );
}

#[test]
#[should_panic(expected = "Jump Targets Broken, target = None")]
fn negative_test_6() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_6_testfail",
    );
}

#[test]
#[should_panic(expected = "assertion failed: heap_safe")]
fn negative_test_7() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_7_testfail",
    );
}

#[test]
#[should_panic(expected = "assertion failed: heap_safe")]
fn negative_test_8() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_8_testfail",
    );
}

#[test]
#[should_panic(expected = "assertion failed: heap_safe")]
fn negative_test_9() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_9_testfail",
    );
}

#[test]
#[should_panic(expected = "assertion failed: heap_safe")]
fn negative_test_10() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_10_testfail",
    );
}

#[test]
#[should_panic(expected = "assertion failed: heap_safe")]
fn negative_test_11() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_11_testfail",
    );
}

#[test]
#[should_panic(expected = "not implemented")]
fn negative_test_12() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_12_testfail",
    );
}

#[test]
#[should_panic(expected = "not implemented")]
fn negative_test_13() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_13_testfail",
    );
}

#[test]
#[should_panic(expected = "assertion failed: stack_safe")]
fn negative_test_14() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_14_testfail",
    );
}

// # NaCl issue #23
#[test]
#[should_panic(expected = "assertion failed: call_safe")]
fn negative_test_nacl_23() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_nacl_23",
    );
}

#[test]
#[should_panic(expected = "not implemented")]
fn negative_test_nacl_323_1() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_nacl_323_1",
    );
}

#[test]
#[should_panic(expected = "not implemented")]
fn negative_test_nacl_323_2() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_nacl_323_2",
    );
}

#[test]
#[should_panic(expected = "not implemented")]
fn negative_test_nacl_323_3() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_nacl_323_3",
    );
}

#[test]
#[should_panic(expected = "assertion failed: stack_safe")]
fn negative_test_nacl_323_4() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_nacl_323_4",
    );
}

#[test]
#[should_panic(expected = "not implemented")]
fn negative_test_nacl_390() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_nacl_390",
    );
}

#[test]
#[should_panic(expected = "Illegal RSP access")]
fn negative_test_nacl_1585() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_nacl_1585",
    );
}

#[test]
#[should_panic(expected = "not implemented")]
fn negative_test_nacl_2532() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_nacl_2532",
    );
}

#[test]
#[should_panic]
fn negative_test_bakersfield_1() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_bakersfield_1",
    );
}

#[test]
#[should_panic(expected = "assertion failed: stack_safe")]
fn negative_test_misfit_1() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_misfit_1",
    );
}

#[test]
#[should_panic(expected = "Jump Targets Broken, target = None")]
fn negative_test_cranelift_805() {
    negative_test_helper(
        "veriwasm_public_data/negative_tests/negative_tests.so",
        "guest_func_cranelift_805",
    );
}
