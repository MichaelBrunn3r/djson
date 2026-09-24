//! Benches string parsing

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use djson::from_bytes;

mod common;

fn parse_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_str");

    for (name, can_borrow) in [
        ("str_short_1k", true),
        ("str_escaped_1k", false),
        ("str_utf16_escapes_1k", false),
        ("str_mixed_1k", false),
    ] {
        let src = common::read_bench_resource(&format!("{name}.dj"));

        if can_borrow {
            group.bench_with_input(
                BenchmarkId::new("borrowed", name),
                &src,
                |benchmark, src| {
                    benchmark.iter(|| {
                        let values: Vec<&str> =
                            from_bytes(black_box(src)).expect("document parses");
                        black_box(values)
                    });
                },
            );
        } else {
            group.bench_with_input(BenchmarkId::new("owned", name), &src, |benchmark, src| {
                benchmark.iter(|| {
                    let values: Vec<String> = from_bytes(black_box(src)).expect("document parses");
                    black_box(values)
                });
            });
        }
    }

    group.finish();
}

criterion_group!(benches, parse_str);
criterion_main!(benches);
