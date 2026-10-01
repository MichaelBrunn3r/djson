//! Benches integer parsing across fixed digit counts.

use criterion::{Criterion, criterion_group, criterion_main};

mod utils;

const NUM_DIGITS: [u32; 9] = [1, 3, 4, 7, 9, 10, 18, 19, 20];
const NUM_DIGITS_WITH_SEP: [u32; 8] = [1, 3, 4, 9, 10, 18, 19, 20];
const I64_MAX_DIGITS: u32 = 18;

fn sep_fixture(digits: u32) -> String {
    // We assume separated groups of 3 digits, so for <= 3 digits, the files are the same.
    let dir: &str = if digits <= 3 {
        "num_digits"
    } else {
        "num_digits_sep"
    };
    format!("int/{dir}/{digits}_30kB.dj")
}

fn parse_int_sweep(c: &mut Criterion) {
    let mut uint_group = c.benchmark_group("parse_int_sweep/uint");
    for digits in NUM_DIGITS {
        utils::bench_from_bytes::<Vec<u64>>(
            &mut uint_group,
            &format!("int/num_digits/{digits}_30kB.dj"),
        );
    }
    uint_group.finish();

    let mut uint_sep_group = c.benchmark_group("parse_int_sweep/uint_sep");
    for digits in NUM_DIGITS_WITH_SEP {
        utils::bench_from_bytes::<Vec<u64>>(&mut uint_sep_group, &sep_fixture(digits));
    }
    uint_sep_group.finish();

    let mut sint_group = c.benchmark_group("parse_int_sweep/sint");
    for digits in NUM_DIGITS
        .into_iter()
        .filter(|&digits| digits <= I64_MAX_DIGITS)
    {
        utils::bench_from_bytes::<Vec<i64>>(
            &mut sint_group,
            &format!("int/num_digits/{digits}_30kB.dj"),
        );
    }
    sint_group.finish();

    let mut sint_sep_group = c.benchmark_group("parse_int_sweep/sint_sep");
    for digits in NUM_DIGITS_WITH_SEP
        .into_iter()
        .filter(|&digits| digits <= I64_MAX_DIGITS)
    {
        utils::bench_from_bytes::<Vec<i64>>(&mut sint_sep_group, &sep_fixture(digits));
    }
    sint_sep_group.finish();
}

criterion_group!(benches, parse_int_sweep);
criterion_main!(benches);
