import { range } from "../range.ts";
import { intWriter, listWriter, structWriter } from "../value_writer.ts";
import type { Recipe } from "./mod.ts";

const one = intWriter(range(1, 1));

const struct = structWriter({
    id: [one, false],
    age: [one, false],
    name: [one, false],
    score: [one, false],
});

/** A flat list of structs with short field names */
export const recipe: Recipe = {
    root: listWriter(() => struct, range(1000, 1000)),
    seed: 44n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};
