import { fixedProvider } from "../provider.ts";
import { range } from "../range.ts";
import { floatWriter, listWriter } from "../value_writer.ts";
import type { Recipe } from "./mod.ts";

/**
 * A flat list of floats
 */
export const recipe: Recipe = {
    root: listWriter(
        fixedProvider(
            floatWriter(range(-10_000_000.0, 10_000_000.0), range(1, 8)),
        ),
        range(1000, 1000),
    ),
    seed: 45n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};
