use criterion::{Criterion, criterion_group, criterion_main};

mod utils;

fn parse_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_uint");

    utils::bench_from_bytes::<Vec<u64>>(&mut group, "u64/short_5k.dj");
    utils::bench_from_bytes::<Vec<u64>>(&mut group, "u64/mixed_5k.dj");

    group.finish();
}

criterion_group!(benches, parse_u64);
criterion_main!(benches);
