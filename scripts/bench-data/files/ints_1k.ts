import { fixedProvider } from "../provider.ts";
import { range } from "../range.ts";
import { intWriter, listWriter } from "../value_writer.ts";
import type { Recipe } from "./mod.ts";

/** A flat list of integers */
export const recipe: Recipe = {
    root: listWriter(
        fixedProvider(intWriter(range(-10_000_000, 10_000_000))),
        range(1000, 1000),
    ),
    seed: 44n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};
