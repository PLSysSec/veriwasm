# veriwasm
veriwasm, but now in Rust!  

To Setup:  
`git submodule update --init --recursive`  
`cargo build  `

To Run:  
`cargo run -- -i <input path>  `

To Test:  
`git clone git@github.com:PLSysSec/veriwasm_data.git`  
`cd veriwasm_data && sh setup.sh && sh build_negative_tests.sh && cd ..`  
`cargo test`  

