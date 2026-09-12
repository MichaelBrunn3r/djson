{
  description = "DJSON language, CLI, and VS Code extension";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    crane,
    ...
  }: let
    system = "x86_64-linux";
    lib = nixpkgs.lib;
    pkgs = (nixpkgs.legacyPackages.${system}).extend rust-overlay.overlays.default;

    craneLib = (crane.mkLib pkgs).overrideToolchain pkgs.rust-bin.nightly.latest.minimal;
    craneWorkspaceRoot = lib.cleanSourceWith {
      src = craneLib.path ./.;
      filter = path: type:
        craneLib.filterCargoSources path type
        || (builtins.match ".*/pkgs/[^/]+/src/.*" (toString path) != null);
      name = "source";
    };
    craneWorkspaceDeps = craneLib.buildDepsOnly {
      src = craneWorkspaceRoot;
      pname = "djson";
      version = "0.1.0";
    };
    djson = craneLib.buildPackage {
      src = craneWorkspaceRoot;
      cargoArtifacts = craneWorkspaceDeps;
      pname = "djson";
      version = "0.1.0";
      cargoExtraArgs = "--locked -p djson-cli";
      meta.mainProgram = "djson";
    };
  in {
    packages.${system}.default = djson;
    checks.${system}.default = djson;
  };
}
