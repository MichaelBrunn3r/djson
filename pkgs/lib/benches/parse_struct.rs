//! Benches struct parsing.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use djson::from_bytes;

mod utils;

fn parse_struct(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_struct");

    let short = utils::read_bench_resource("struct_short_keys_1k.dj");
    group.bench_with_input(
        BenchmarkId::from_parameter("struct_short_keys_1k"),
        &short,
        |benchmark, src| {
            benchmark.iter(|| {
                let values: Vec<ShortKeys> = from_bytes(black_box(src)).expect("document parses");
                black_box(values)
            });
        },
    );

    let long = utils::read_bench_resource("struct_long_keys_1k.dj");
    group.bench_with_input(
        BenchmarkId::from_parameter("struct_long_keys_1k"),
        &long,
        |benchmark, src| {
            benchmark.iter(|| {
                let values: Vec<CatStats> = from_bytes(black_box(src)).expect("document parses");
                black_box(values)
            });
        },
    );

    let partial = utils::read_bench_resource("struct_partial_1k.dj");
    group.bench_with_input(
        BenchmarkId::from_parameter("struct_partial_1k"),
        &partial,
        |benchmark, src| {
            benchmark.iter(|| {
                let values: Vec<Partial> = from_bytes(black_box(src)).expect("document parses");
                black_box(values)
            });
        },
    );

    group.finish();
}

criterion_group!(benches, parse_struct);
criterion_main!(benches);

#[allow(dead_code)]
#[derive(serde_derive::Deserialize)]
struct ShortKeys {
    id: u64,
    age: u64,
    name: u64,
    score: u64,
}

#[allow(dead_code)]
#[derive(serde_derive::Deserialize)]
struct CatStats {
    successful_laser_dot_captures: u64,
    total_minutes_spent_sleeping: u64,
    objects_pushed_off_ledge_count: u64,
    equivalent_volume_in_milliliters: u64,
}

#[allow(dead_code)]
#[derive(serde_derive::Deserialize)]
struct Partial {
    a: Option<u64>,
    b: Option<u64>,
    c: Option<u64>,
    d: Option<u64>,
    e: Option<u64>,
    f: Option<u64>,
    g: Option<u64>,
}
