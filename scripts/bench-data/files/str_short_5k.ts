import { range } from "../range.ts";
import {
  BYTE_DOUBLE_QUOTE,
  delimitedWriter,
  listWriter,
  stringWriter,
} from "../value_writer.ts";
import { flatten, weighted } from "../weight_tree.ts";
import { WeightedChoices } from "../weighted_choices.ts";
import type { Recipe } from "./mod.ts";

const stringWriters = new WeightedChoices(
  flatten([
    delimitedWriter(
      stringWriter({ lenBounds: range(1, 3) }),
      [BYTE_DOUBLE_QUOTE],
    ),
    weighted(
      2,
      delimitedWriter(
        stringWriter({ lenBounds: range(4, 8) }),
        [BYTE_DOUBLE_QUOTE],
      ),
    ),
  ]),
);

/** A flat list of strings */
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
