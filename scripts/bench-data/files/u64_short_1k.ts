import { range } from "../range.ts";
import { intWriter, listWriter } from "../value_writer.ts";
import { flatten } from "../weight_tree.ts";
import { WeightedChoices } from "../weighted_choices.ts";
import type { Recipe } from "./mod.ts";

const integerWriters = new WeightedChoices(
  flatten([
    intWriter(range(0, 9)),
    intWriter(range(10, 99)),
    intWriter(range(100, 999)),
    intWriter(range(1000, 9999)),
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
