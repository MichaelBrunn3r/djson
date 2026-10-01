import type { ValueWriter } from "../value_writer.ts";
import { recipe as str_short_5k_recipe } from "./str/short_5k.ts";
import { recipe as str_mixed_5k_recipe } from "./str/mixed_5k.ts";
import { recipe as str_escaped_5k_recipe } from "./str/escaped_5k.ts";
import { recipe as str_utf16_escapes_5k_recipe } from "./str/utf16_escapes_5k.ts";
import { recipe as int_mixed_5k_recipe } from "./int/mixed_5k.ts";
import {
    num_digits_10_30kB_recipe,
    num_digits_18_30kB_recipe,
    num_digits_19_30kB_recipe,
    num_digits_1_30kB_recipe,
    num_digits_20_30kB_recipe,
    num_digits_3_30kB_recipe,
    num_digits_4_30kB_recipe,
    num_digits_7_30kB_recipe,
    num_digits_9_30kB_recipe,
} from "./int/num_digits.ts";
import {
    num_digits_sep_10_30kB_recipe,
    num_digits_sep_18_30kB_recipe,
    num_digits_sep_19_30kB_recipe,
    num_digits_sep_20_30kB_recipe,
    num_digits_sep_4_30kB_recipe,
    num_digits_sep_9_30kB_recipe,
} from "./int/num_digits_sep.ts";
import { recipe as struct_short_keys_1k_recipe } from "./struct/short_keys_1k.ts";
import { recipe as struct_long_keys_1k_recipe } from "./struct/long_keys_1k.ts";
import { recipe as struct_partial_1k_recipe } from "./struct/partial_1k.ts";
import {
    len_keys_16_100kB_recipe,
    len_keys_1_100kB_recipe,
    len_keys_32_100kB_recipe,
    len_keys_8_100kB_recipe,
} from "./struct/len_keys.ts";
import {
    num_fields_16_100kB_recipe,
    num_fields_1_100kB_recipe,
    num_fields_32_100kB_recipe,
    num_fields_8_100kB_recipe,
} from "./struct/num_fields.ts";
import { recipe as utf16_escapes_1k_recipe } from "./utf16_escapes_1k.ts";

export const BENCHMARK_FILES: Readonly<Record<string, BenchmarkFile>> = {
    // region String
    str_short_5k: {
        path: "pkgs/lib/benches/res/str/short_5k.dj",
        recipe: str_short_5k_recipe,
    },
    str_escaped_5k: {
        path: "pkgs/lib/benches/res/str/escaped_5k.dj",
        recipe: str_escaped_5k_recipe,
    },
    str_mixed_5k: {
        path: "pkgs/lib/benches/res/str/mixed_5k.dj",
        recipe: str_mixed_5k_recipe,
    },
    str_utf16_escapes_5k: {
        path: "pkgs/lib/benches/res/str/utf16_escapes_5k.dj",
        recipe: str_utf16_escapes_5k_recipe,
    },
    // endregion String
    // region int
    int_mixed_5k: {
        path: "pkgs/lib/benches/res/int/mixed_5k.dj",
        recipe: int_mixed_5k_recipe,
    },
    // endregion int
    // region int: sweep fixtures
    num_digits_1_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits/1_30kB.dj",
        recipe: num_digits_1_30kB_recipe,
    },
    num_digits_3_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits/3_30kB.dj",
        recipe: num_digits_3_30kB_recipe,
    },
    num_digits_4_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits/4_30kB.dj",
        recipe: num_digits_4_30kB_recipe,
    },
    num_digits_7_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits/7_30kB.dj",
        recipe: num_digits_7_30kB_recipe,
    },
    num_digits_9_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits/9_30kB.dj",
        recipe: num_digits_9_30kB_recipe,
    },
    num_digits_10_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits/10_30kB.dj",
        recipe: num_digits_10_30kB_recipe,
    },
    num_digits_18_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits/18_30kB.dj",
        recipe: num_digits_18_30kB_recipe,
    },
    num_digits_19_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits/19_30kB.dj",
        recipe: num_digits_19_30kB_recipe,
    },
    num_digits_20_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits/20_30kB.dj",
        recipe: num_digits_20_30kB_recipe,
    },
    // endregion int: sweep fixtures
    // region int: separator sweep fixtures
    num_digits_sep_4_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits_sep/4_30kB.dj",
        recipe: num_digits_sep_4_30kB_recipe,
    },
    num_digits_sep_9_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits_sep/9_30kB.dj",
        recipe: num_digits_sep_9_30kB_recipe,
    },
    num_digits_sep_10_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits_sep/10_30kB.dj",
        recipe: num_digits_sep_10_30kB_recipe,
    },
    num_digits_sep_18_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits_sep/18_30kB.dj",
        recipe: num_digits_sep_18_30kB_recipe,
    },
    num_digits_sep_19_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits_sep/19_30kB.dj",
        recipe: num_digits_sep_19_30kB_recipe,
    },
    num_digits_sep_20_30kB: {
        path: "pkgs/lib/benches/res/int/num_digits_sep/20_30kB.dj",
        recipe: num_digits_sep_20_30kB_recipe,
    },
    // endregion int: separator sweep fixtures
    // region struct
    struct_short_keys_1k: {
        path: "pkgs/lib/benches/res/struct/short_keys_1k.dj",
        recipe: struct_short_keys_1k_recipe,
    },
    struct_long_keys_1k: {
        path: "pkgs/lib/benches/res/struct/long_keys_1k.dj",
        recipe: struct_long_keys_1k_recipe,
    },
    struct_partial_1k: {
        path: "pkgs/lib/benches/res/struct/partial_1k.dj",
        recipe: struct_partial_1k_recipe,
    },
    // endregion struct
    // region struct: sweep fixtures
    len_keys_1_100kB: {
        path: "pkgs/lib/benches/res/struct/len_keys/1_100kB.dj",
        recipe: len_keys_1_100kB_recipe,
    },
    len_keys_8_100kB: {
        path: "pkgs/lib/benches/res/struct/len_keys/8_100kB.dj",
        recipe: len_keys_8_100kB_recipe,
    },
    len_keys_16_100kB: {
        path: "pkgs/lib/benches/res/struct/len_keys/16_100kB.dj",
        recipe: len_keys_16_100kB_recipe,
    },
    len_keys_32_100kB: {
        path: "pkgs/lib/benches/res/struct/len_keys/32_100kB.dj",
        recipe: len_keys_32_100kB_recipe,
    },
    num_fields_1_100kB: {
        path: "pkgs/lib/benches/res/struct/num_fields/1_100kB.dj",
        recipe: num_fields_1_100kB_recipe,
    },
    num_fields_8_100kB: {
        path: "pkgs/lib/benches/res/struct/num_fields/8_100kB.dj",
        recipe: num_fields_8_100kB_recipe,
    },
    num_fields_16_100kB: {
        path: "pkgs/lib/benches/res/struct/num_fields/16_100kB.dj",
        recipe: num_fields_16_100kB_recipe,
    },
    num_fields_32_100kB: {
        path: "pkgs/lib/benches/res/struct/num_fields/32_100kB.dj",
        recipe: num_fields_32_100kB_recipe,
    },
    // endregion struct: sweep fixtures
    utf16_escapes_1k: {
        path: "pkgs/lib/benches/res/utf16_escapes_1k.txt",
        recipe: utf16_escapes_1k_recipe,
    },
};

export interface BenchmarkFile {
    /** Destination path, relative to the repository root. */
    readonly path: string;
    /** How the document is generated. */
    readonly recipe: Recipe;
}

export interface Recipe {
    /** The root of the document. */
    readonly root: ValueWriter;
    readonly seed: bigint;
    /**
     * The document is written until it holds at least this many bytes.
     *
     * This is the writer's byte budget, so it is a cap rather than a floor: a
     * recipe whose root has a fixed shape must not set it too low, or the
     * document is cut short. Such recipes use `Infinity` and terminate on
     * their own; only a recipe that grows until the budget runs out needs a
     * finite value, which is what stops it.
     */
    readonly minBytes: number;
    /** Target line width. */
    readonly lineWidth: number;
    /** How deep values may nest before shallow values are used instead. */
    readonly maxDepth: number;
}
