{pkgs, ...}: {
  languages.rust = {
    enable = true;
    channel = "nightly";
    components = ["rustc" "cargo" "clippy" "rustfmt" "rust-analyzer"];
  };

  packages = with pkgs; [
    cargo-insta
    deno
    git
    perf
    samply
    vsce
  ];
}
