use clap::{App, Arg};
use loaders::types::{ExecutableType, VwArch};
use std::str::FromStr;
use veriwasm::loaders;
use veriwasm::runner::*;

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
        .arg(
            Arg::with_name("architecture")
                .long("arch")
                .takes_value(true)
                .help("Architecture of the executable (x64 | aarch64)"),
        )
        .arg(Arg::with_name("disable_stack_checks").long("disable_stack_checks"))
        .arg(Arg::with_name("disable_linear_mem_checks").long("disable_linear_mem_checks"))
        .arg(Arg::with_name("disable_call_checks").long("disable_call_checks"))
        .arg(Arg::with_name("enable_zero_cost_checks").long("enable_zero_cost_checks"))
        .get_matches();

    let module_path = matches.value_of("module path").unwrap();
    let num_jobs_opt = matches.value_of("jobs");
    let output_path = matches.value_of("stats output path").unwrap_or("");
    let num_jobs = num_jobs_opt
        .map(|s| s.parse::<u32>().unwrap_or(1))
        .unwrap_or(1);
    let disable_stack_checks = matches.is_present("disable_stack_checks");
    let disable_linear_mem_checks = matches.is_present("disable_linear_mem_checks");
    let disable_call_checks = matches.is_present("disable_call_checks");
    let enable_zero_cost_checks = matches.is_present("enable_zero_cost_checks");
    let only_func = matches.value_of("one function").map(|s| s.to_owned());
    let executable_type =
        ExecutableType::from_str(matches.value_of("executable type").unwrap_or("lucet")).unwrap();
    let arch = VwArch::from_str(matches.value_of("architecture").unwrap_or("x64")).unwrap();

    let has_output = if output_path == "" { false } else { true };

    let active_passes = PassConfig {
        stack: !disable_stack_checks,
        linear_mem: !disable_linear_mem_checks,
        call: !disable_call_checks,
        zero_cost: enable_zero_cost_checks,
    };

    let config = Config {
        module_path: module_path.to_string(),
        _num_jobs: num_jobs,
        output_path: output_path.to_string(),
        has_output: has_output,
        only_func,
        executable_type,
        active_passes,
        arch,
    };

    run(config);
}
