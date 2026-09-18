import { fixedProvider } from "../provider.ts";
import { range } from "../range.ts";
import { exprWriter, floatWriter, listWriter } from "../value_writer.ts";
import type { Recipe } from "./mod.ts";

// Keep operands in safe bounds to avoid overflows.
const OPERAND_BOUNDS = range(-50_000.0, 50_000.0);

/**
 * A flat list of float expressions.
 */
export const recipe: Recipe = {
    root: listWriter(
        fixedProvider(
            exprWriter(
                fixedProvider(floatWriter(OPERAND_BOUNDS, range(1, 8))),
                range(2, 4),
            ),
        ),
        range(1000, 1000),
    ),
    seed: 48n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};
