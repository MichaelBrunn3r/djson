//! Helpers shared by the benchmark binaries.

use std::{fs, path::PathBuf};

/// Reads the document named `name` from the bench resource directory.
pub fn read_bench_resource(name: &str) -> Vec<u8> {
    let path = resolve_bench_resource(name);
    fs::read(&path).unwrap_or_else(|error| panic!("reading `{}`: {error}", path.display()))
}

/// Absolute path of `name` in the bench resource directory.
pub fn resolve_bench_resource(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches/res")
        .join(name)
}
