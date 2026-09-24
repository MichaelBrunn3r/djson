//! Compares implementations of decoding `\uXXXX` to a u16.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

mod utils;

fn compare_parse_hex4(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_hex4");

    let src = utils::read_bench_resource("utf16_escapes_1k.txt");
    let src = src.strip_suffix(b"\n").unwrap_or(&src); // the generator added a \n
    let num_escapes = src.len() / "\\uXXXX".len();

    for &(impl_name, parse) in &IMPLS {
        group.bench_with_input(impl_name, &num_escapes, |benchmark, &num_escapes| {
            benchmark.iter(|| {
                for i in 0..num_escapes {
                    let offset = i * "\\uXXXX".len() + "\\u".len(); // start at the i-th escape after the '\u'
                    black_box(parse(black_box(src), offset));
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, compare_parse_hex4);
criterion_main!(benches);

const IMPLS: [(&str, fn(&[u8], usize) -> Option<u16>); 2] =
    [("scalar", parse_scalar), ("lut", parse_with_lut)];

fn parse_scalar(src: &[u8], pos: usize) -> Option<u16> {
    let hex: &[u8; 4] = src.get(pos..pos + 4)?.try_into().ok()?;

    let mut unit = 0;

    for &byte in hex {
        let digit = char::from(byte)
            .to_digit(16)
            .and_then(|digit| u16::try_from(digit).ok())?;
        unit = unit * 16 + digit;
    }

    Some(unit)
}

//region Impl LUT
fn parse_with_lut(src: &[u8], pos: usize) -> Option<u16> {
    let hex: &[u8; 4] = src.get(pos..pos + 4)?.try_into().ok()?;

    let digits = hex.map(|byte| HEX_LUT[usize::from(byte)]);
    let unit = (digits[0] << 12) | (digits[1] << 8) | (digits[2] << 4) | digits[3];

    u16::try_from(unit).ok()
}

/// Maps all bytes in '0-9a-fA-F' to their respective nibble value.
/// All other bytes are mapped to `u32::MAX`.
const HEX_LUT: [u32; 256] = {
    let mut lut = [u32::MAX; 256];

    let mut digit = 0u8;
    while digit < 10 {
        lut[(b'0' + digit) as usize] = digit as u32;
        digit += 1;
    }

    let mut letter = 0u8;
    while letter < 6 {
        let digit = (10 + letter) as u32;
        lut[(b'a' + letter) as usize] = digit;
        lut[(b'A' + letter) as usize] = digit;
        letter += 1;
    }

    lut
};

//endregion
