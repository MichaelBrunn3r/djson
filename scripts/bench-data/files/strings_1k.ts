import type { ValueWriterProvider } from "../provider.ts";
import { range } from "../range.ts";
import {
    delimitedWriter,
    DOUBLE_QUOTE,
    listWriter,
    randomWordWriter,
    SINGLE_QUOTE,
    stringWriter,
    type ValueWriter,
} from "../value_writer.ts";
import { WeightedChoices } from "../weighted_choices.ts";
import { flatten, weighted } from "../weight_tree.ts";
import { words } from "../words.ts";
import type { Recipe } from "./mod.ts";

const QUOTES = [SINGLE_QUOTE, DOUBLE_QUOTE];

const mixProvider: ValueWriterProvider = (rng, _stopNesting) =>
    stringKinds.choose(rng.nextFloat());

/**
 * A flat list quoted strings.
 */
export const recipe: Recipe = {
    root: listWriter(mixProvider, range(1000, 1000)),
    seed: 46n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};

const stringKinds = new WeightedChoices(
    flatten<ValueWriter>([
        delimitedWriter(stringWriter({ lenBounds: range(1, 16) }), QUOTES),
        weighted(
            2,
            delimitedWriter(
                randomWordWriter(words, { countBounds: range(1, 6) }),
                QUOTES,
            ),
        ),
    ]),
);
