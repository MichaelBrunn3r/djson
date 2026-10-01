import { intRange, range } from "../../range.ts";
import {
    bigIntWriter,
    intWriter,
    listWriter,
    type ValueWriter,
} from "../../value_writer.ts";
import type { Recipe } from "../mod.ts";

function numDigitsRecipe(writer: ValueWriter): Recipe {
    return {
        root: listWriter(() => writer),
        seed: 44n,
        minBytes: 30_000,
        lineWidth: 100,
        maxDepth: 0,
    };
}

export const num_digits_1_30kB_recipe = numDigitsRecipe(intWriter(range(0, 9)));

export const num_digits_3_30kB_recipe = numDigitsRecipe(
    intWriter(range(100, 999)),
);

export const num_digits_4_30kB_recipe = numDigitsRecipe(
    intWriter(range(1_000, 9_999)),
);

export const num_digits_7_30kB_recipe = numDigitsRecipe(
    intWriter(range(1_000_000, 9_999_999)),
);

export const num_digits_9_30kB_recipe = numDigitsRecipe(
    intWriter(range(100_000_000, 999_999_999)),
);

export const num_digits_10_30kB_recipe = numDigitsRecipe(
    intWriter(range(1_000_000_000, 9_999_999_999)),
);

export const num_digits_18_30kB_recipe = numDigitsRecipe(
    bigIntWriter(intRange(100_000_000_000_000_000n, 999_999_999_999_999_999n)),
);

export const num_digits_19_30kB_recipe = numDigitsRecipe(
    bigIntWriter(
        intRange(1_000_000_000_000_000_000n, 9_999_999_999_999_999_999n),
    ),
);

export const num_digits_20_30kB_recipe = numDigitsRecipe(
    bigIntWriter(
        intRange(10_000_000_000_000_000_000n, 18_446_744_073_709_551_615n),
    ),
);
