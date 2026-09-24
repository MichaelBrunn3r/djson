use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use djson::from_bytes;

mod common;

fn parse_u64(c: &mut Criterion) {
    for name in ["u64_short_1k", "u64_mixed_1k"] {
        let src = common::read_bench_resource(&format!("{name}.dj"));

        c.bench_function(name, |benchmark| {
            benchmark.iter(|| {
                let values: Vec<u64> = from_bytes(black_box(&src)).expect("example parses");
                black_box(values)
            });
        });
    }
}

criterion_group!(benches, parse_u64);
criterion_main!(benches);
