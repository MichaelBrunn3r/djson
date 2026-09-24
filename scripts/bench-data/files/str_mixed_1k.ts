import {
    ESCAPE_SEQUENCES,
    escapedStringWriter,
    PAIR_UNICODE_ESCAPES,
    pickByWeight,
    SINGLE_UNICODE_ESCAPES,
} from "../djson_utils.ts";
import { range } from "../range.ts";
import { listWriter } from "../value_writer.ts";
import { flatten, weighted } from "../weight_tree.ts";
import { WeightedChoices } from "../weighted_choices.ts";
import type { Recipe } from "./mod.ts";

const escapeChance = 0.02;
const pickEscape = pickByWeight([
    weighted(4, ESCAPE_SEQUENCES),
    weighted(2, SINGLE_UNICODE_ESCAPES),
    weighted(1, PAIR_UNICODE_ESCAPES),
]);

const stringWriters = new WeightedChoices(
    flatten([
        weighted(
            1,
            escapedStringWriter(range(1, 4), { escapeChance, pickEscape }),
        ),
        weighted(
            4,
            escapedStringWriter(range(5, 10), { escapeChance, pickEscape }),
        ),
        weighted(
            1,
            escapedStringWriter(range(9, 32), { escapeChance, pickEscape }),
        ),
    ]),
);

export const recipe: Recipe = {
    root: listWriter(
        (rng) => rng.choose(stringWriters),
        range(5000, 5000),
    ),
    seed: 44n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};
