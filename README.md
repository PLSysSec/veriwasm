# veriwasm
veriwasm, but now in Rust!  

WebAssembly (Wasm) is a platform-independent bytecode that offers both good performance and runtime isolation. To implement isolation, the compiler inserts safety checks when it compiles Wasm to native machine code. While this approach is cheap, it also requires trust in the compiler's correctness---trust that the compiler has inserted each necessary check, correctly formed, in each proper place. Unfortunately, subtle bugs in the Wasm compiler can break---and have broken---isolation guarantees. To address this problem, we propose verifying memory isolation of Wasm binaries post-compilation. We implement this approach in VeriWasm, a static offline verifier for native x86-64 binaries compiled from Wasm; we prove the verifier's soundness, and find that it can detect bugs with no false positives. Finally, we describe our deployment of VeriWasm at Fastly.

To Setup:  
`git submodule update --init --recursive`  
`cargo build --release  `

To Run:  
`cargo run --release -- -i <input path> -o <output path for statistics> `

To Test:  
`git clone git@github.com:PLSysSec/veriwasm_data.git`  
`cd veriwasm_data && sh setup.sh && sh build_negative_tests.sh && cd ..`  
`cargo test --release`  

