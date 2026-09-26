use criterion::{Criterion, criterion_group, criterion_main};

mod utils;

fn parse_struct_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_struct_sweep");

    // region Num Fields=5, Key length=1..32
    utils::bench_from_bytes::<Vec<LenKeys1>>(&mut group, "struct/len_keys/1_100kB.dj");
    utils::bench_from_bytes::<Vec<LenKeys8>>(&mut group, "struct/len_keys/8_100kB.dj");
    utils::bench_from_bytes::<Vec<LenKeys16>>(&mut group, "struct/len_keys/16_100kB.dj");
    utils::bench_from_bytes::<Vec<LenKeys32>>(&mut group, "struct/len_keys/32_100kB.dj");
    // endregion

    // region Num Fields=1..32, Key length=8
    utils::bench_from_bytes::<Vec<NumFields1>>(&mut group, "struct/num_fields/1_100kB.dj");
    utils::bench_from_bytes::<Vec<NumFields8>>(&mut group, "struct/num_fields/8_100kB.dj");
    utils::bench_from_bytes::<Vec<NumFields16>>(&mut group, "struct/num_fields/16_100kB.dj");
    utils::bench_from_bytes::<Vec<NumFields32>>(&mut group, "struct/num_fields/32_100kB.dj");
    // endregion

    group.finish();
}

criterion_group!(benches, parse_struct_sweep);
criterion_main!(benches);

//region Type for deserialization
#[allow(dead_code)]
#[derive(serde_derive::Deserialize)]
struct LenKeys1 {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
}

#[allow(dead_code)]
#[derive(serde_derive::Deserialize)]
struct LenKeys8 {
    key_0001: u64,
    key_0002: u64,
    key_0003: u64,
    key_0004: u64,
    key_0005: u64,
}

#[allow(dead_code)]
#[derive(serde_derive::Deserialize)]
struct LenKeys16 {
    key_field_000001: u64,
    key_field_000002: u64,
    key_field_000003: u64,
    key_field_000004: u64,
    key_field_000005: u64,
}

#[allow(dead_code)]
#[derive(serde_derive::Deserialize)]
struct LenKeys32 {
    key_field_thirty_two_bytes_00001: u64,
    key_field_thirty_two_bytes_00002: u64,
    key_field_thirty_two_bytes_00003: u64,
    key_field_thirty_two_bytes_00004: u64,
    key_field_thirty_two_bytes_00005: u64,
}

#[allow(dead_code)]
#[derive(serde_derive::Deserialize)]
struct NumFields1 {
    key_0001: u64,
}

#[allow(dead_code)]
#[derive(serde_derive::Deserialize)]
struct NumFields8 {
    key_0001: u64,
    key_0002: u64,
    key_0003: u64,
    key_0004: u64,
    key_0005: u64,
    key_0006: u64,
    key_0007: u64,
    key_0008: u64,
}

#[allow(dead_code)]
#[derive(serde_derive::Deserialize)]
struct NumFields16 {
    key_0001: u64,
    key_0002: u64,
    key_0003: u64,
    key_0004: u64,
    key_0005: u64,
    key_0006: u64,
    key_0007: u64,
    key_0008: u64,
    key_0009: u64,
    key_0010: u64,
    key_0011: u64,
    key_0012: u64,
    key_0013: u64,
    key_0014: u64,
    key_0015: u64,
    key_0016: u64,
}

#[allow(dead_code)]
#[derive(serde_derive::Deserialize)]
struct NumFields32 {
    key_0001: u64,
    key_0002: u64,
    key_0003: u64,
    key_0004: u64,
    key_0005: u64,
    key_0006: u64,
    key_0007: u64,
    key_0008: u64,
    key_0009: u64,
    key_0010: u64,
    key_0011: u64,
    key_0012: u64,
    key_0013: u64,
    key_0014: u64,
    key_0015: u64,
    key_0016: u64,
    key_0017: u64,
    key_0018: u64,
    key_0019: u64,
    key_0020: u64,
    key_0021: u64,
    key_0022: u64,
    key_0023: u64,
    key_0024: u64,
    key_0025: u64,
    key_0026: u64,
    key_0027: u64,
    key_0028: u64,
    key_0029: u64,
    key_0030: u64,
    key_0031: u64,
    key_0032: u64,
}

//endregion
