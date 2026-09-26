import { range } from "../range.ts";
import { intWriter, listWriter, structWriter } from "../value_writer.ts";
import type { Recipe } from "./mod.ts";

const one = intWriter(range(1, 1));

const struct = structWriter({
    successful_laser_dot_captures: [one, false],
    total_minutes_spent_sleeping: [one, false],
    objects_pushed_off_ledge_count: [one, false],
    equivalent_volume_in_milliliters: [one, false],
});

/** A flat list of structs with long field names */
export const recipe: Recipe = {
    root: listWriter(() => struct, range(1000, 1000)),
    seed: 44n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};
