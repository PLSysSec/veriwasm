FROM ubuntu:latest

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update
RUN apt-get install -y curl unzip git make cmake m4 python3 wget nasm
RUN apt update
RUN apt install build-essential -y


# Rust dependencies to compile program to wasm
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install cargo-wasi


# Install wasm32-wasi
RUN rustup target add wasm32-wasi

# Clone and build veriwasm docker branch
RUN git clone https://github.com/PLSysSec/veriwasm.git 
WORKDIR /veriwasm/
RUN git checkout docker
RUN make bootstrap
RUN cargo build --release


# This will setup fuzzers, and by doing so, build clang and lucet
RUN make build_fuzzers

# Load binaries to test
RUN make build_public_data

# Add shortcuts for compilers to compile your own sandboxed applications
RUN cat enable_compilers >> /root/.bashrc

