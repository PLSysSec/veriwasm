pub mod lattices;
pub mod utils;
use utils::{get_cfgs};
use std::env;


fn run(binpath : &str){
    // let cfgs = get_cfgs(binpath);
    for cfg in get_cfgs(binpath).iter(){
        //TODO: check instruction legality
        println!("Checking Instruction Legality");
        //TODO: check stack safety
        println!("Checking Stack Safety");
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

