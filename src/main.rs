pub mod lattices;
pub mod utils;
pub mod analyses;
use utils::{get_cfgs};
use std::env;
use analyses::stack_analyzer::{analyze_stack};



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
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);
    run(&args[1]);

}

