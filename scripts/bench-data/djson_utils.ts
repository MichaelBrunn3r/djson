import { assert } from "jsr:@std/assert";
import { type Range, range } from "./range.ts";
import { type Random } from "./random.ts";
import { flatten, type WeightTree } from "./weight_tree.ts";
import { WeightedChoices } from "./weighted_choices.ts";
import {
    alternationWriter,
    BYTE_DOUBLE_QUOTE,
    delimitedWriter,
    type SequencePicker,
    stringWriter,
    type ValueWriter,
} from "./value_writer.ts";
import type { Writer } from "./writer.ts";

const encoder = new TextEncoder();

/** Every escape sequence a djson string may contain, excluding `\uXXXX`. */
export const ESCAPE_SEQUENCES: readonly Uint8Array[] = [
    '\\"', // \"
    "\\\\", // \\
    "\\/", // \/
    "\\b", // \b
    "\\f", // \f
    "\\n", // \n
    "\\r", // \r
    "\\t", // \t
].map((escape) => encoder.encode(escape));

/** A list of valid single surrogates. */
export const SINGLE_UNICODE_ESCAPES: readonly Uint8Array[] = [
    "\\u0041", // A
    "\\u007f", // DELETE
    "\\u00e9", // é
    "\\u03c0", // π
    "\\u20ac", // €
    "\\u30ad", // キ
].map((escape) => encoder.encode(escape));

/** A list of valid surrogate pairs. */
export const PAIR_UNICODE_ESCAPES: readonly Uint8Array[] = [
    "\\uD83D\\uDE00", // 😀
    "\\uD83D\\uDE03", // 😃
    "\\uD83C\\uDF89", // 🎉
    "\\uD83D\\uDD25", // 🔥
].map((escape) => encoder.encode(escape));

export function pickByWeight(tree: WeightTree<Uint8Array>): SequencePicker {
    assert(tree.length > 0, "expected at least one byte sequence");
    const choices = new WeightedChoices(flatten(tree));
    return (rng: Random) => rng.choose(choices);
}

export const LONE_CODE_UNITS: readonly Range[] = [
    range(0x0000, 0xd7ff),
    range(0xe000, 0xffff),
];

export const PAIR_CODE_UNITS: readonly [Range, Range] = [
    range(0xd800, 0xdbff),
    range(0xdc00, 0xdfff),
];

const UTF16_ESCAPE_LENGTH = 6;
const UTF16_ESCAPE_PREFIX = encoder.encode("\\u");
const HEX_LOWERCASE_DIGITS = encoder.encode("0123456789abcdef");
const HEX_UPPERCASE_DIGITS = encoder.encode("0123456789ABCDEF");

export function createUtf16Escape(unit: number, rng: Random): Uint8Array {
    const digits = rng.nextFloat() < 0.5
        ? HEX_LOWERCASE_DIGITS
        : HEX_UPPERCASE_DIGITS;
    const escape = new Uint8Array(UTF16_ESCAPE_LENGTH);
    escape.set(UTF16_ESCAPE_PREFIX);
    for (let i = 0; i < 4; i++) {
        escape[2 + i] = digits[(unit >>> (12 - 4 * i)) & 0xf];
    }
    return escape;
}

export function pickUtf16Escape(): SequencePicker {
    return (rng: Random) =>
        createUtf16Escape(rng.int(rng.pick(LONE_CODE_UNITS)), rng);
}

export function pickUtf16EscapePair(): SequencePicker {
    return (rng: Random) => {
        const [leading, trailing] = PAIR_CODE_UNITS;
        const pair = new Uint8Array(2 * UTF16_ESCAPE_LENGTH);
        pair.set(createUtf16Escape(rng.int(leading), rng));
        pair.set(
            createUtf16Escape(rng.int(trailing), rng),
            UTF16_ESCAPE_LENGTH,
        );
        return pair;
    };
}

export function escapeStreamWriter(
    pickEscape: SequencePicker,
    count: number,
): ValueWriter {
    return {
        write(rng: Random, writer: Writer): void {
            let escapes = 0;
            while (escapes < count) {
                const escape = pickEscape(rng);
                writer.writeBytes(escape);
                escapes += escape.length / UTF16_ESCAPE_LENGTH;
            }
        },
    };
}

/** Writes a delimited string of letters, mixed in with escape sequences. */
export function escapedStringWriter(
    lengthBounds: Range,
    {
        escapeChance = 0.05,
        atLeastOne = false,
        pickEscape = pickByWeight(ESCAPE_SEQUENCES),
    }: {
        escapeChance?: number;
        atLeastOne?: boolean;
        pickEscape?: SequencePicker;
    } = {},
): ValueWriter {
    return delimitedWriter(
        alternationWriter(
            stringWriter({ lenBounds: range(1, 1) }),
            pickEscape,
            {
                countBounds: lengthBounds,
                sequenceChance: escapeChance,
                atLeastOne,
            },
        ),
        [BYTE_DOUBLE_QUOTE],
    );
}
