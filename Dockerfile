FROM ubuntu:latest

RUN apt-get update
RUN apt-get install -y curl unzip git make
RUN apt update
RUN apt install build-essential -y



# Rust dependencies to compile program to wasm
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install cargo-wasi


# Install wasm32-wasi
RUN rustup target add wasm32-wasi


#COPY . /veriwasm/
RUN git clone https://github.com/PLSysSec/veriwasm.git
WORKDIR /veriwasm/

RUN cargo build --release

# This will setup fuzzers, and by doing so, build clang and lucet
#RUN make build_fuzzers

# add instructions for compiling your own c or rust code to wasm

