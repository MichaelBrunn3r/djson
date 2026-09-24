//! Benches string parsing

use std::hint::black_box;

use bumpalo::{Bump, collections::Vec as ArenaVec};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use djson::{Deserializer, from_bytes};
use serde::de::DeserializeSeed;
use serde_state::de::SeqSeed;

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
            // group.bench_with_input(BenchmarkId::new("owned", name), &src, |benchmark, src| {
            //     benchmark.iter(|| {
            //         let values: Vec<String> = from_bytes(black_box(src)).expect("document parses");
            //         black_box(values)
            //     });
            // });

            group.bench_with_input(BenchmarkId::new("owned", name), &src, |benchmark, src| {
                benchmark.iter(|| {
                    let arena = Bump::new();
                    let mut deserializer = Deserializer::new(black_box(src));
                    let values: ArenaVec<'_, &str> =
                        SeqSeed::new(common::ArenaString { arena: &arena }, |capacity| {
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
