import { range } from "../../range.ts";
import { intWriter, listWriter, structWriter } from "../../value_writer.ts";
import type { Recipe } from "../mod.ts";

const one = intWriter(range(1, 1));

function lenKeysRecipe(keys: readonly string[]): Recipe {
    const struct = structWriter(
        Object.fromEntries(keys.map((key) => [key, [one, false]] as const)),
    );
    return {
        root: listWriter(() => struct),
        seed: 44n,
        minBytes: 100_000,
        lineWidth: 100,
        maxDepth: 0,
    };
}

export const len_keys_1_100kB_recipe = lenKeysRecipe(["a", "b", "c", "d", "e"]);

export const len_keys_8_100kB_recipe = lenKeysRecipe([
    "key_0001",
    "key_0002",
    "key_0003",
    "key_0004",
    "key_0005",
]);

export const len_keys_16_100kB_recipe = lenKeysRecipe([
    "key_field_000001",
    "key_field_000002",
    "key_field_000003",
    "key_field_000004",
    "key_field_000005",
]);

export const len_keys_32_100kB_recipe = lenKeysRecipe([
    "key_field_thirty_two_bytes_00001",
    "key_field_thirty_two_bytes_00002",
    "key_field_thirty_two_bytes_00003",
    "key_field_thirty_two_bytes_00004",
    "key_field_thirty_two_bytes_00005",
]);
