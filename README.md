# VeriWasm
This repository contains all the code and data necessary for building VeriWasm and reproducing the results presented in [our paper](http://cseweb.ucsd.edu/~dstefan/pubs/johnson:2021:veriwasm.pdf).  
  
## Abstract  
WebAssembly (Wasm) is a platform-independent bytecode that offers both good performance and runtime isolation. To implement isolation, the compiler inserts safety checks when it compiles Wasm to native machine code. While this approach is cheap, it also requires trust in the compiler's correctness---trust that the compiler has inserted each necessary check, correctly formed, in each proper place. Unfortunately, subtle bugs in the Wasm compiler can break---and have broken---isolation guarantees. To address this problem, we propose verifying memory isolation of Wasm binaries post-compilation. We implement this approach in VeriWasm, a static offline verifier for native x86-64 binaries compiled from Wasm; we prove the verifier's soundness, and find that it can detect bugs with no false positives. Finally, we describe our deployment of VeriWasm at Fastly.

## Reproducing Evaluation Results
We provide the infrastructure to reproduce the results in the paper here. 

First, install prequisites:

### VeriWasm build prequisites

- git
- Rust
- nasm (to compile test cases)
- gcc (to compile test cases)

To Setup:  
`git submodule update --init --recursive`  
`cargo build --release  `

### Running the evaluation suite
This verifies all binaries used in the paper, with the exception on the Spec2006 binaries (Spec2006 is proprietary) and the Fastly production binaries.

To test:  
`git clone https://github.com/PLSysSec/veriwasm_public_data.git`  
`cd veriwasm_public_data && sh setup.sh && sh build_negative_tests.sh && cd ..`  
`cargo test --release`  

### Getting Performance statistics
These commands get the performance statistics for the binaries (besides Spec2006 and the Fastly production binaries). 

To get stats:  
  `make compute_stats`  
  `python3 graph_stats.py stats/*`  



### Running VeriWasm on your own binaries

To run:  
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

### Fuzzing VeriWasm
First, install prequisites:

- python3 (for scripts)
- cmake  

Then, build the fuzzers (and the tooling they rely on).

To build:  
  `make build_fuzzers`  

Then, either run the Csmith-based fuzzer or the Wasm-based fuzzer. The make command used 4 cores by default.

To run Csmith fuzzer:  
  `cd veriwasm_fuzzing`  
  `make csmith_fuzz`
  
To run Wasm fuzzer:  
  `cd veriwasm_fuzzing`  
  `make wasm_fuzz`  

## Related repos

### Binaries used for evaluation
The binaries we verified as part of our evaluation our in a seperate repo, located [here](https://github.com/PLSysSec/veriwasm_public_data.git).

### Fuzzing scripts
The scripts we used to fuzz VeriWasm are located [here](https://github.com/PLSysSec/veriwasm_fuzzing).

### Mechanized proofs
The proofs from our paper are in a seperate repo, located [here](https://github.com/PLSysSec/veriwasm-verification).
