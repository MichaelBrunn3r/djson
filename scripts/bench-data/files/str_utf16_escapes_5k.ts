import {
    escapedStringWriter,
    PAIR_UNICODE_ESCAPES,
    pickByWeight,
    SINGLE_UNICODE_ESCAPES,
} from "../djson_utils.ts";
import { type Range, range } from "../range.ts";
import {
    listWriter,
    type SequencePicker,
    type ValueWriter,
} from "../value_writer.ts";
import { flatten, weighted } from "../weight_tree.ts";
import { WeightedChoices } from "../weighted_choices.ts";
import type { Recipe } from "./mod.ts";

const singleEscapes = pickByWeight(SINGLE_UNICODE_ESCAPES);
const pairEscapes = pickByWeight(PAIR_UNICODE_ESCAPES);

function unicodeStringWriter(
    pickEscape: SequencePicker,
    lengthBounds: Range,
    escapeChance: number,
): ValueWriter {
    return escapedStringWriter(lengthBounds, {
        pickEscape,
        escapeChance,
        atLeastOne: true,
    });
}

const stringWriters = new WeightedChoices(
    flatten([
        // Single
        weighted(4, unicodeStringWriter(singleEscapes, range(4, 16), 0.02)),
        weighted(1, unicodeStringWriter(singleEscapes, range(24, 48), 0.02)),
        // Pairs
        weighted(2, unicodeStringWriter(pairEscapes, range(2, 8), 0.01)),
        weighted(1, unicodeStringWriter(pairEscapes, range(12, 24), 0.01)),
    ]),
);

export const recipe: Recipe = {
    root: listWriter(
        (rng) => rng.choose(stringWriters),
        range(5000, 5000),
    ),
    seed: 46n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};
