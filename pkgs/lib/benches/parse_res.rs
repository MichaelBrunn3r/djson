use std::{fs, hint::black_box, path::PathBuf};

use criterion::{Criterion, criterion_group, criterion_main};
use djson::from_bytes;

fn parse_u64(c: &mut Criterion) {
    for (name, resource) in [
        ("u64_short_1k", "u64_short_1k.dj"),
        ("u64_mixed_1k", "u64_mixed_1k.dj"),
    ] {
        let path = resolve_bench_resource(resource);
        let src = read_bench_resource(&path);

        c.bench_function(name, |benchmark| {
            benchmark.iter(|| {
                let values: Vec<u64> = from_bytes(black_box(&src)).expect("example parses");
                black_box(values)
            });
        });
    }
}

fn parse_str(c: &mut Criterion) {
    let path = resolve_bench_resource("str_1k.dj");
    let src = read_bench_resource(&path);

    c.bench_function("str_1k", |benchmark| {
        benchmark.iter(|| {
            // The strings hold no escapes, so they borrow from the source.
            let values: Vec<&str> = from_bytes(black_box(&src)).expect("example parses");
            black_box(values)
        });
    });
}

criterion_group!(benches, parse_u64, parse_str);
criterion_main!(benches);

fn read_bench_resource(path: &PathBuf) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|error| panic!("reading `{}`: {error}", path.display()))
}

fn resolve_bench_resource(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches/res")
        .join(name)
}
