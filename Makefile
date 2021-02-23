build_fuzzers:
	git clone https://github.com/PLSysSec/veriwasm_fuzzing.git
	cd veriwasm_fuzzing && make build

compute_stats:
	if [ ! -d ./stats ]; then \
                mkdir stats; \
        fi
	cargo run --release -- -i veriwasm_data/firefox_libs/liboggwasm.so  -o stats/liboggwasm.stats &
	cargo run --release -- -i veriwasm_data/firefox_libs/libgraphitewasm.so -o stats/libgraphitewasm.stats

compute_stats_all:
	if [ ! -d ./stats ]; then \
                mkdir stats; \
        fi
	cargo run --release -- -i veriwasm_data/firefox_libs/liboggwasm.so  -o stats/liboggwasm.stats &
	cargo run --release -- -i veriwasm_data/firefox_libs/libgraphitewasm.so -o stats/libgraphitewasm.stats &
	cargo run --release -- -i veriwasm_data/shootout/shootout.so -o stats/shootout.stats

