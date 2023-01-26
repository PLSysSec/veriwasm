build_fuzzers:
	git clone git@github.com:PLSysSec/veriwasm_fuzzing.git
	cd veriwasm_fuzzing && make build

compute_stats:
	if [ ! -d ./stats ]; then \
                mkdir stats; \
        fi
	cargo run --release -- -i veriwasm_public_data/firefox_libs/liboggwasm.so  -o stats/liboggwasm.stats &
	cargo run --release -- -i veriwasm_public_data/firefox_libs/libgraphitewasm.so -o stats/libgraphitewasm.stats

compute_stats_all:
	if [ ! -d ./stats ]; then \
                mkdir stats; \
        fi
	cargo run --release -- -i veriwasm_data/firefox_libs/liboggwasm.so  -o stats/liboggwasm.stats &
	cargo run --release -- -i veriwasm_data/firefox_libs/libgraphitewasm.so -o stats/libgraphitewasm.stats &
	cargo run --release -- -i veriwasm_data/shootout/shootout.so -o stats/shootout.stats

compute_stats_zerocost:
	mkdir -p ./zerocost_stats
	cargo build --release 
	target/release/veriwasm -i veriwasm_public_data/zerocost_bins/graphiteogghunspell.so -o zerocost_stats/graphiteogghunspell
	target/release/veriwasm -i veriwasm_public_data/zerocost_bins/soundtouch.so -o zerocost_stats/soundtouch
	target/release/veriwasm -i veriwasm_public_data/zerocost_bins/libexpatwasm.so -o zerocost_stats/libexpatwasm
	target/release/veriwasm -i veriwasm_public_data/zerocost_spec_libraries/astar.so -o zerocost_stats/astar
	target/release/veriwasm -i veriwasm_public_data/zerocost_spec_libraries/bzip2.so -o zerocost_stats/bzip2
	target/release/veriwasm -i veriwasm_public_data/zerocost_spec_libraries/gobmk.so -o zerocost_stats/gobmk
	target/release/veriwasm -i veriwasm_public_data/zerocost_spec_libraries/h264ref.so -o zerocost_stats/h264ref
	target/release/veriwasm -i veriwasm_public_data/zerocost_spec_libraries/lbm.so -o zerocost_stats/lbm
	target/release/veriwasm -i veriwasm_public_data/zerocost_spec_libraries/libquantum.so -o zerocost_stats/libquantum
	target/release/veriwasm -i veriwasm_public_data/zerocost_spec_libraries/mcf.so -o zerocost_stats/mcf
	target/release/veriwasm -i veriwasm_public_data/zerocost_spec_libraries/milc.so -o zerocost_stats/milc
	target/release/veriwasm -i veriwasm_public_data/zerocost_spec_libraries/namd.so -o zerocost_stats/namd
	target/release/veriwasm -i veriwasm_public_data/zerocost_spec_libraries/sjeng.so -o zerocost_stats/sjeng

build_public_data:
	git clone git@github.com:PLSysSec/veriwasm_public_data.git
	cd veriwasm_public_data && bash setup.sh && bash build_negative_tests.sh && bash build_wasmtime_tests.sh 

bootstrap:
	git clone https://github.com/PLSysSec/lucet_sandbox_compiler.git
	cd lucet_sandbox_compiler && git submodule update --init --recursive && cargo build --release


