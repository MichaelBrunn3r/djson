set positional-arguments
target_dir := env_var_or_default('CARGO_TARGET_DIR', './target')

profile bin:
    cargo build --profile profiling -p {{ bin }}
    samply record -- {{ target_dir }}/profiling/{{ bin }}

build-vscode-ext:
    cd pkgs/vscode-ext && deno task build && vsce package --no-dependencies

init *args:
    deno run --allow-read --allow-write --allow-net scripts/init.ts {{ args }}

# just plot uint_num_digits sint_num_digits
# just plot --y MB/s --groups "1,2;3,4" uint_sep sint_sep uint_num_digits sint_num_digits
plot *args:
    uv run scripts/plot_benches/main.py "$@"
