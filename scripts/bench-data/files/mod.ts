import type { ValueWriter } from "../value_writer.ts";
import { recipe as exprsRecipe } from "./exprs_1MB.ts";
import { recipe as floatExprsRecipe } from "./float_exprs_1k.ts";
import { recipe as floatsRecipe } from "./floats_1k.ts";
import { recipe as intExprsRecipe } from "./int_exprs_1k.ts";
import { recipe as intsRecipe } from "./ints_1k.ts";
import { recipe as staticRecipe } from "./static_1MB.ts";
import { recipe as stringsRecipe } from "./strings_1k.ts";

export const BENCHMARK_FILES: Readonly<Record<string, BenchmarkFile>> = {
    static_1MB: {
        path: "pkgs/lib/benches/generated/static_1MB.dj",
        recipe: staticRecipe,
    },
    exprs_1MB: {
        path: "pkgs/lib/benches/generated/exprs_1MB.dj",
        recipe: exprsRecipe,
    },
    ints_1k: {
        path: "pkgs/lib/benches/generated/ints_1k.dj",
        recipe: intsRecipe,
    },
    floats_1k: {
        path: "pkgs/lib/benches/generated/floats_1k.dj",
        recipe: floatsRecipe,
    },
    strings_1k: {
        path: "pkgs/lib/benches/generated/strings_1k.dj",
        recipe: stringsRecipe,
    },
    int_exprs_1k: {
        path: "pkgs/lib/benches/generated/int_exprs_1k.dj",
        recipe: intExprsRecipe,
    },
    float_exprs_1k: {
        path: "pkgs/lib/benches/generated/float_exprs_1k.dj",
        recipe: floatExprsRecipe,
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

export const HASHES_FILE = "scripts/hashes.json";
const HASHES_URL = new URL("../../hashes.json", import.meta.url);
export type Hashes = Readonly<Record<string, string>>;

export async function load_file_hashes(): Promise<Hashes> {
    try {
        return JSON.parse(await Deno.readTextFile(HASHES_URL));
    } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
            return {};
        }
        throw error;
    }
}

export async function save_file_hashes(hashes: Hashes): Promise<void> {
    const sorted = Object.fromEntries(
        Object.entries(hashes).sort(([left], [right]) =>
            left.localeCompare(right)
        ),
    );
    await Deno.writeTextFile(
        HASHES_URL,
        `${JSON.stringify(sorted, null, 2)}\n`,
    );
}
