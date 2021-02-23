# VeriWasm: SFI safety for native-compiled Wasm

This repository contains all the code and data necessary for building VeriWasm and reproducing the results presented in our NDSS'21 paper [Доверя́й, но проверя́й: SFI safety for native-compiled Wasm](http://cseweb.ucsd.edu/~dstefan/pubs/johnson:2021:veriwasm.pdf).  
  
## Abstract  
WebAssembly (Wasm) is a platform-independent bytecode that offers both good performance and runtime isolation. To implement isolation, the compiler inserts safety checks when it compiles Wasm to native machine code. While this approach is cheap, it also requires trust in the compiler's correctness—trust that the compiler has inserted each necessary check, correctly formed, in each proper place. Unfortunately, subtle bugs in the Wasm compiler can break—and have broken—isolation guarantees. To address this problem, we propose verifying memory isolation of Wasm binaries post-compilation. We implement this approach in VeriWasm, a static offline verifier for native x86-64 binaries compiled from Wasm; we prove the verifier's soundness, and find that it can detect bugs with no false positives. Finally, we describe our deployment of VeriWasm at Fastly.

## Build VeriWasm

You first need to install several dependencies:

- git
- Rust
- nasm (to compile test cases)
- gcc (to compile test cases)
- python3 (for scripts)

Once you have these, you can build VeriWasm:

```bash
git submodule update --init --recursive
cargo build --release
```

## Run VeriWasm

To run VeriWasm on your own binaries, you just need to point it to the module you want to check:

```bash
cargo run --release -- -i <input path> 
```

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

## Reproducing Evaluation Results

This repo contains all the infrastructure necessary for reproducing the results described in the paper. Once you build VeriWasm you can run our tests and and performance benchmarks.

### Running the evaluation suite

To verify all the binaries described in the paper, except the SPEC CPU 2006 binaries (they are proprietary) and the Fastly production binaries, run:

```bash
git clone https://github.com/PLSysSec/veriwasm_public_data.git
cd veriwasm_public_data && sh setup.sh && sh build_negative_tests.sh && cd ..
cargo test --release
```

To get get the performance statistics for the binaries, run:

```bash
make compute_stats
python3 graph_stats.py stats/*
```

### Fuzzing VeriWasm

To fuzz VeriWasm, you'll need to install `cmake` and then build the fuzzers (and the tooling they rely on):

```bash
make build_fuzzers
```

Then, either run the  [Csmith](https://embed.cs.utah.edu/csmith/)-based fuzzer:

```bash
cd veriwasm_fuzzing
make csmith_fuzz
```

or the Wasm-based fuzzer:    

```bash
cd veriwasm_fuzzing
make wasm_fuzz
```

By default, `make` will use four cores; you may want to change this.

## Related repos

- **Binaries used for evaluation**: The binaries we verified as part of our evaluation are [here](https://github.com/PLSysSec/veriwasm_public_data.git).

- **Fuzzing scripts**: The scripts we used to fuzz VeriWasm are [here](https://github.com/PLSysSec/veriwasm_fuzzing).

- **Mechanized proofs**: The proofs from our paper are [here](https://github.com/PLSysSec/veriwasm-verification).
