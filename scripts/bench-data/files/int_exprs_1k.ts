import { fixedProvider } from "../provider.ts";
import { range } from "../range.ts";
import { exprWriter, intWriter, listWriter } from "../value_writer.ts";
import type { Recipe } from "./mod.ts";

// Keep operands in safe bounds to avoid overflows.
const OPERAND_BOUNDS = range(-50_000, 50_000);

/**
 * A flat list of int expressions.
 */
export const recipe: Recipe = {
    root: listWriter(
        fixedProvider(
            exprWriter(
                fixedProvider(intWriter(OPERAND_BOUNDS)),
                range(2, 4),
            ),
        ),
        range(1000, 1000),
    ),
    seed: 47n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};
