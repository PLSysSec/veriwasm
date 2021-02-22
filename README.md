# VeriWasm
This repository contains all the code and data necessary for building VeriWasm and reproducing the results presented in [our paper](http://cseweb.ucsd.edu/~dstefan/pubs/johnson:2021:veriwasm.pdf).  
  
## Abstract  
WebAssembly (Wasm) is a platform-independent bytecode that offers both good performance and runtime isolation. To implement isolation, the compiler inserts safety checks when it compiles Wasm to native machine code. While this approach is cheap, it also requires trust in the compiler's correctness---trust that the compiler has inserted each necessary check, correctly formed, in each proper place. Unfortunately, subtle bugs in the Wasm compiler can break---and have broken---isolation guarantees. To address this problem, we propose verifying memory isolation of Wasm binaries post-compilation. We implement this approach in VeriWasm, a static offline verifier for native x86-64 binaries compiled from Wasm; we prove the verifier's soundness, and find that it can detect bugs with no false positives. Finally, we describe our deployment of VeriWasm at Fastly.

## Reproducing Evaluation Results


First, install prequisites:

### VeriWasm Build Prequisites

- git
- Rust
- nasm (to compile test cases)
- gcc (to compile test cases)

To Setup:  
`git submodule update --init --recursive`  
`cargo build --release  `

To Run:  
`cargo run --release -- -i <input path> `

Usage:  

```
VeriWasm 0.1.0
Validates safety of native Wasm code

USAGE:
    veriwasm [FLAGS] [OPTIONS] -i <module path>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      
    -V, --version    Prints version information

OPTIONS:
    -j, --jobs <jobs>                   Number of parallel threads (default 1)
    -i <module path>                    path to native Wasm module to validate
    -o, --output <stats output path>    Path to output stats file

```

To Test:  
`git clone git@github.com:PLSysSec/veriwasm_public_data.git`  
`cd veriwasm_public_data && sh setup.sh && sh build_negative_tests.sh && cd ..`  
`cargo test --release`  

### VeriWasm Fuzzing Prequisites

- python3 (for scripts)
- csmith (to produce random C files)
- clang (to compile csmith-generated files to Wasm)
- binaryen (to produce random Wasm files)
- lucet compiler (To compile Wasm to native code)

## Repos

### Binaries Used for Evaluation
The binaries we verified as part of our evaluation our in a seperate repo, located [here](https://github.com/PLSysSec/veriwasm_public_data.git).

### Fuzzing Scripts

### Mechanized Proofs
The proofs from our paper are in a seperate repo, located [here](https://github.com/PLSysSec/veriwasm-verification).
