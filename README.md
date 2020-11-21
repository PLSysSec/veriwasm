# veriwasm
veriwasm, but now in Rust!

To Setup:
git submodule update --init --recursive  
cargo build

To Run:
cargo run -- -i <input path>

To Test:
git clone git@github.com:PLSysSec/veriwasm\_data.git
sh veriwasm\_data/build.sh
cargo test

