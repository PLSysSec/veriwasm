pub mod lattices;
pub mod utils;
pub mod analyses;
use utils::{get_cfgs};
use std::env;
use analyses::stack_analyzer::{analyze_stack};
use clap::{Arg, App};


fn run(binpath : &str){
    // let cfgs = get_cfgs(binpath);
    for cfg in get_cfgs(binpath).iter(){
        //TODO: check instruction legality
        println!("Checking Instruction Legality");
        //TODO: check stack safety
        println!("Checking Stack Safety");
        let stack_result = analyze_stack(cfg);
        //TODO: check heap safety
        println!("Checking Heap Safety");
        //TODO: check call safety
        println!("Checking Call Safety");

    }
    println!("Done!");
}

fn main() {
    // let args: Vec<String> = env::args().collect();
    // // println!("{:?}", args);
    // run(&args[1]);
    let matches = App::new("VeriWasm")
    .version("0.1.0")
    .about("Validates safety of native Wasm module")
    // .arg(Arg::with_name("command")
    //     .short("cmd")
    //     .long("command")
    //     .takes_value(true)
    //     .help("A cool file"))
    .arg(Arg::with_name("module path")
        .short("i")
        .takes_value(true)
        .help("path to native Wasm module to validate")
        .required(true))
    .arg(Arg::with_name("jobs")
        .short("j")
        .long("jobs")
        .takes_value(true)
        .help("Number of parallel threads"))
    .arg(Arg::with_name("stats output path")
        .short("o")
        .long("output")
        .takes_value(true)
        .help("Path to output stats file"))
    .arg(Arg::with_name("quiet")
        .short("q")
        .long("quiet")
        .takes_value(true))
    .get_matches();

    run(matches.value_of("module path").unwrap());

}

