compute_stats:
		 [ ! -d ./stats ]; then \
                mkdir stats; \
        fi
        cargo run --release -- -i veriwasm_data/firefox_libs/liboggwasm.so  -o stats/liboggwasm.stats &
        cargo run --release -- -i veriwasm_data/firefox_libs/libgraphitewasm.so -o stats/libgraphitewasm.stats &		
        cargo run --release -- -i veriwasm_data/spec/astar_base.wasm_lucet  -o stats/astar.stats &
        cargo run --release -- -i veriwasm_data/spec/gobmk_base.wasm_lucet  -o stats/gobmk.stats &
        cargo run --release -- -i veriwasm_data/spec/lbm_base.wasm_lucet    -o stats/lbm.stats   &
        cargo run --release -- -i veriwasm_data/spec/mcf_base.wasm_lucet    -o stats/mcf.stats   &
        cargo run --release -- -i veriwasm_data/spec/namd_base.wasm_lucet   -o stats/namd.stats  &
        cargo run --release -- -i veriwasm_data/spec/sjeng_base.wasm_lucet  -o stats/sjeng.stats &
        cargo run --release -- -i veriwasm_data/spec/sphinx_livepretend_base.wasm_lucet -o stats/sphinx.stats &
        cargo run --release -- -i veriwasm_data/spec/bzip2_base.wasm_lucet  -o stats/bzip2.stats &
        cargo run --release -- -i veriwasm_data/spec/h264ref_base.wasm_lucet  -o stats/h264ref.stats &
        cargo run --release -- -i veriwasm_data/spec/libquantum_base.wasm_lucet -o stats/libquantum.stats &
        cargo run --release -- -i veriwasm_data/spec/milc_base.wasm_lucet  -o stats/milc.stats &
        cargo run --release -- -i veriwasm_data/spec/povray_base.wasm_lucet  -o stats/povray.stats &
        cargo run --release -- -i veriwasm_data/spec/soplex_base.wasm_lucet  -o stats/soplex.stats
