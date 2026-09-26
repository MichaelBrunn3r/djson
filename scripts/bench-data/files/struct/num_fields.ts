import { range } from "../../range.ts";
import { intWriter, listWriter, structWriter } from "../../value_writer.ts";
import type { Recipe } from "../mod.ts";

const one = intWriter(range(1, 1));

function keys(count: number): string[] {
    return Array.from(
        { length: count },
        (_, index) => `key_${String(index + 1).padStart(4, "0")}`,
    );
}

function numFieldsRecipe(count: number): Recipe {
    const struct = structWriter(
        Object.fromEntries(
            keys(count).map((key) => [key, [one, false]] as const),
        ),
    );
    return {
        root: listWriter(() => struct),
        seed: 44n,
        minBytes: 100_000,
        lineWidth: 100,
        maxDepth: 0,
    };
}

export const num_fields_1_100kB_recipe = numFieldsRecipe(1);

export const num_fields_8_100kB_recipe = numFieldsRecipe(8);

export const num_fields_16_100kB_recipe = numFieldsRecipe(16);

export const num_fields_32_100kB_recipe = numFieldsRecipe(32);
