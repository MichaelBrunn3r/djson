import { assert } from "jsr:@std/assert";
import type { Random } from "./random.ts";
import { intWriter, type ValueWriter } from "./value_writer.ts";
import { WeightedChoices } from "./weighted_choices.ts";
import { type Range } from "./range.ts";

export type Catalogue = Record<string, WeightedChoices<ValueWriter>>;

/** Every type a value can have. */
export const VALUE_TYPES = [
    "int",
    "float",
    "bool",
    "string",
    "null",
    "list",
    "map",
] as const satisfies readonly string[];

export type ValueType = (typeof VALUE_TYPES)[number];

/**
 * Supplies a random writer on each invocation.
 *
 * `stopNesting` indicates that the writer has chosen to stop nesting any further.
 * The provider is obligated to return a writer that does not nest further.
 */
export type ValueWriterProvider = (
    rng: Random,
    stopNesting: boolean,
) => ValueWriter;

/**
 * Picks a random value writer from a catalogue on each invocation.
 * Picks from `nonNesting` instead whenever `stopNesting` is true.
 */
export function anyValueProvider(
    catalogue: Catalogue,
    nonNesting?: Catalogue,
): ValueWriterProvider {
    return (rng: Random, stopNesting: boolean): ValueWriter => {
        const source = stopNesting && nonNesting ? nonNesting : catalogue;
        return rng.choose(source[rng.pick(VALUE_TYPES)]);
    };
}

/**
 * Picks a random writer from the given `key` in `catalogue`.
 * Picks from `nonNesting` instead whenever `stopNesting` is true.
 */
export function typeProvider(
    key: ValueType | "key",
    catalogue: Catalogue,
    nonNesting?: Catalogue,
): ValueWriterProvider {
    return (rng: Random, stopNesting: boolean): ValueWriter => {
        const source = stopNesting && nonNesting && nonNesting[key]
            ? nonNesting
            : catalogue;
        return rng.choose(source[key]);
    };
}

/** Supplies the same terminal `writer` on every invocation. */
export function fixedProvider(writer: ValueWriter): ValueWriterProvider {
    return () => writer;
}

/** Supplies a plain positive integer from `bounds` on each invocation. */
export function countProvider(bounds: Range): ValueWriterProvider {
    assert(
        bounds.min > 0,
        `count bounds must be positive, got min: ${bounds.min}`,
    );
    return () => intWriter(bounds);
}
