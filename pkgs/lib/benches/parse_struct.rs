//! Benches struct parsing.

use criterion::{Criterion, criterion_group, criterion_main};

mod utils;

fn parse_struct(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_struct");

    utils::bench_from_bytes::<Vec<ShortKeys>>(&mut group, "struct/short_keys_1k.dj");
    utils::bench_from_bytes::<Vec<CatStats>>(&mut group, "struct/long_keys_1k.dj");
    utils::bench_from_bytes::<Vec<Partial>>(&mut group, "struct/partial_1k.dj");

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
