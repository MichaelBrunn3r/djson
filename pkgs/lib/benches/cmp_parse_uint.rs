//! Compares djson uint parsing against other impls

use std::hint::black_box;

use criterion::{
    BenchmarkGroup, BenchmarkId, Criterion, criterion_group, criterion_main, measurement::WallTime,
};
use djson::from_bytes;
use rand::{RngExt, SeedableRng as _, rngs::StdRng};

fn parse_integers(c: &mut Criterion) {
    let count = 512;
    // 1-4 digits
    let nums_short = gen_nums((0, 9999), count, 1);
    // 13-19 digits
    let nums_long = gen_nums((1_000_000_000_000, 9_999_999_999_999_999_999), count, 2);
    // 20 - 24 digits overflow u64
    let nums_overflow = gen_nums(
        (10_000_000_000_000_000_000, 99_999_999_999_999_999_999_999),
        count,
        3,
    );
    let mixed: Vec<String> = (0..count)
        .flat_map(|index| [&nums_short[index], &nums_long[index], &nums_overflow[index]])
        .cloned()
        .collect();

    let mut group = c.benchmark_group("parse_u64");
    bench_nums(&mut group, "short", &nums_short);
    bench_nums(&mut group, "long", &nums_long);
    bench_nums(&mut group, "overflow", &nums_overflow);
    bench_nums(&mut group, "mixed", &mixed);
    group.finish();
}

fn bench_nums(group: &mut BenchmarkGroup<'_, WallTime>, name: &str, nums: &[String]) {
    group.bench_with_input(
        BenchmarkId::new("djson", name),
        nums,
        |benchmark, integers| {
            benchmark.iter(|| {
                for integer in integers {
                    black_box(from_bytes::<u64>(black_box(integer.as_bytes())).ok());
                }
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("stdlib", name),
        nums,
        |benchmark, integers| {
            benchmark.iter(|| {
                for integer in integers {
                    black_box(integer.parse::<u64>().ok());
                }
            });
        },
    );
}

fn gen_nums(range: (u128, u128), count: usize, seed: u64) -> Vec<String> {
    let (lowest, highest) = range;
    assert!(lowest <= highest, "range must not be empty");

    let span = highest - lowest + 1;
    let mut rng = StdRng::seed_from_u64(seed);
    (0..count)
        .map(|_| (lowest + rng.random_range(0..span)).to_string())
        .collect()
}

criterion_group!(benches, parse_integers);
criterion_main!(benches);
