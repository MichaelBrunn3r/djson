import { range } from "../range.ts";
import {
    intWriter,
    listWriter,
    nullableWriter,
    structWriter,
} from "../value_writer.ts";
import type { Recipe } from "./mod.ts";

const value = nullableWriter(intWriter(range(1, 1)));

const struct = structWriter({
    a: [value, true],
    b: [value, true],
    c: [value, true],
    d: [value, true],
    e: [value, true],
    f: [value, true],
    g: [value, true],
});

/** A flat list of structs with seven optional fields */
export const recipe: Recipe = {
    root: listWriter(() => struct, range(1000, 1000)),
    seed: 44n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};
