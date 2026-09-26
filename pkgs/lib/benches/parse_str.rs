//! Benches string parsing

use std::hint::black_box;

use bumpalo::{Bump, collections::Vec as ArenaVec};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use djson::{Deserializer, from_bytes};
use serde::de::DeserializeSeed;
use serde_state::de::SeqSeed;

mod utils;

fn parse_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_str");

    for (file, can_borrow) in [
        ("str/short_5k.dj", true),
        ("str/escaped_5k.dj", false),
        ("str/utf16_escapes_5k.dj", false),
        ("str/mixed_5k.dj", false),
    ] {
        let src = utils::read_bench_resource(file);
        let id = utils::resource_id(file);
        group.throughput(Throughput::BytesDecimal(src.len() as u64));

        if can_borrow {
            group.bench_with_input(BenchmarkId::new("borrowed", &id), &src, |benchmark, src| {
                benchmark.iter(|| {
                    let values: Vec<&str> = from_bytes(black_box(src)).expect("document parses");
                    black_box(values)
                });
            });
        } else {
            // Old code that borrows without an arena. Not a scenario I care about.
            // group.bench_with_input(BenchmarkId::new("owned", &id), &src, |benchmark, src| {
            //     benchmark.iter(|| {
            //         let values: Vec<String> = from_bytes(black_box(src)).expect("document parses");
            //         black_box(values)
            //     });
            // });

            group.bench_with_input(BenchmarkId::new("arena", &id), &src, |benchmark, src| {
                benchmark.iter(|| {
                    let arena = Bump::new();
                    let mut deserializer = Deserializer::new(black_box(src));
                    let values: ArenaVec<'_, &str> =
                        SeqSeed::new(utils::ArenaString { arena: &arena }, |capacity| {
                            let mut values = ArenaVec::new_in(&arena);
                            values.reserve(capacity);
                            values
                        })
                        .deserialize(&mut deserializer)
                        .expect("document parses");
                    black_box(values.len());
                });
            });
        }
    }

    group.finish();
}

criterion_group!(benches, parse_str);
criterion_main!(benches);
