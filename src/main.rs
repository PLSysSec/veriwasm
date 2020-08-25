pub mod lattices;
pub mod utils;
pub mod analyses;
pub mod metadata;
pub mod lifter;
pub mod ir_utils;
use utils::{load_program, get_cfgs, load_metadata};
use analyses::stack_analyzer::analyze_stack;
use clap::{Arg, App};
use lifter::lift_cfg;

pub struct Config{
    module_path: String,
    num_jobs: u32,
    output_path : String,
    has_output : bool,
    quiet : bool
}



fn run(config : Config){
    let program = load_program(&config.module_path);
    // let cfgs = get_cfgs(binpath);
    println!("Loading Metadata");
    let metadata = load_metadata(&config.module_path);
    for (func_name,cfg) in get_cfgs(&config.module_path).iter(){
        // let g = &cfg.graph;
        // let blocks = &cfg.blocks;
        // for node in g.nodes(){
        //     let block = cfg.get_block(node);
        //     let mut iter = program.instructions_spanning(<AMD64 as Arch>::Decoder::default(), block.start, block.end);
        //     while let Some((address, instr)) = iter.next() {
        //         lift(instr);
        //         println!("{:?}\n", instr);
        //     }
            
        // }

        println!("Analyzing: {:?}", func_name);
        println!("Checking Instruction Legality");
        let irmap = lift_cfg(&program, cfg);
        println!("Checking Stack Safety");
        let stack_result = analyze_stack(cfg, irmap);
        //TODO: check heap safety
        println!("Checking Heap Safety");
        //TODO: check call safety
        println!("Checking Call Safety");

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

