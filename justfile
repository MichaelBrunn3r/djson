set positional-arguments
target_dir := env_var_or_default('CARGO_TARGET_DIR', './target')

profile bin:
    cargo build --profile profiling -p {{ bin }}
    samply record -- {{ target_dir }}/profiling/{{ bin }}

build-vscode-ext:
    cd pkgs/vscode-ext && deno task build && vsce package --no-dependencies
