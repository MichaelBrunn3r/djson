import { range } from "../range.ts";
import { BYTE_UNDERSCORE, intWriter, listWriter } from "../value_writer.ts";
import { flatten, weighted } from "../weight_tree.ts";
import { WeightedChoices } from "../weighted_choices.ts";
import type { Recipe } from "./mod.ts";

const separators = [BYTE_UNDERSCORE];
const integerWriters = new WeightedChoices(
    flatten([
        // 1-4 digits
        weighted(
            12,
            intWriter(range(0, 9_999), {
                separators: separators,
                separatorChance: 0.1,
            }),
        ),
        // 5-6 digits
        weighted(
            4,
            intWriter(range(10_000, 999_999), {
                separators: separators,
                separatorChance: 0.1,
            }),
        ),
        // 7-9 digits
        weighted(
            2,
            intWriter(range(1_000_000, 999_999_999), {
                separators: separators,
                separatorChance: 0.1,
            }),
        ),
        // 10-12 digits
        weighted(
            1,
            intWriter(range(1_000_000_000, 999_999_999_999), {
                separators: separators,
                separatorChance: 0.1,
            }),
        ),
        // 13-20 digits
        weighted(
            1,
            intWriter(range(1_000_000_000_000, Number.MAX_SAFE_INTEGER), {
                separators: separators,
                separatorChance: 0.1,
            }),
        ),
    ]),
);

/** A flat list of integers */
export const recipe: Recipe = {
    root: listWriter(
        (rng) => rng.choose(integerWriters),
        range(5000, 5000),
    ),
    seed: 44n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};
