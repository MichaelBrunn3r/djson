/**
 * The exact same ints as `num_digits.ts`, but as 3 digit groups, separated by
 * `_`. Values of at most 3 digits are shorter than one group, so they carry no
 * separator and reuse the `num_digits.ts` fixtures instead of a copy here.
 */

import { intRange, range } from "../../range.ts";
import {
    bigIntWriter,
    BYTE_UNDERSCORE,
    type Grouping,
    intWriter,
    listWriter,
    type ValueWriter,
} from "../../value_writer.ts";
import type { Recipe } from "../mod.ts";

const GROUPING: Grouping = { separator: BYTE_UNDERSCORE };

function numDigitsSepRecipe(writer: ValueWriter): Recipe {
    return {
        root: listWriter(() => writer),
        seed: 44n,
        minBytes: 30_000,
        lineWidth: 100,
        maxDepth: 0,
    };
}

export const num_digits_sep_4_30kB_recipe = numDigitsSepRecipe(
    intWriter(range(1_000, 9_999), GROUPING),
);

export const num_digits_sep_9_30kB_recipe = numDigitsSepRecipe(
    intWriter(range(100_000_000, 999_999_999), GROUPING),
);

export const num_digits_sep_10_30kB_recipe = numDigitsSepRecipe(
    intWriter(range(1_000_000_000, 9_999_999_999), GROUPING),
);

export const num_digits_sep_18_30kB_recipe = numDigitsSepRecipe(
    bigIntWriter(
        intRange(100_000_000_000_000_000n, 999_999_999_999_999_999n),
        GROUPING,
    ),
);

export const num_digits_sep_19_30kB_recipe = numDigitsSepRecipe(
    bigIntWriter(
        intRange(1_000_000_000_000_000_000n, 9_999_999_999_999_999_999n),
        GROUPING,
    ),
);

export const num_digits_sep_20_30kB_recipe = numDigitsSepRecipe(
    bigIntWriter(
        intRange(10_000_000_000_000_000_000n, 18_446_744_073_709_551_615n),
        GROUPING,
    ),
);
