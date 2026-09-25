import { range } from "../range.ts";
import { escapedStringWriter } from "../djson_utils.ts";
import { listWriter } from "../value_writer.ts";
import { flatten, weighted } from "../weight_tree.ts";
import { WeightedChoices } from "../weighted_choices.ts";
import type { Recipe } from "./mod.ts";

const stringWriters = new WeightedChoices(
  flatten([
    weighted(
      4,
      escapedStringWriter(range(4, 16), { atLeastOne: true }),
    ),
    weighted(
      1,
      escapedStringWriter(range(24, 48), { atLeastOne: true }),
    ),
  ]),
);

/** A flat list of strings, each holding at least one non-`\u` escape sequence */
export const recipe: Recipe = {
  root: listWriter(
    (rng) => rng.choose(stringWriters),
    range(5000, 5000),
  ),
  seed: 45n,
  minBytes: Infinity,
  lineWidth: 100,
  maxDepth: 0,
};
