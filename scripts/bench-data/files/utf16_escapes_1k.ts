import {
    escapeStreamWriter,
    pickUtf16Escape,
    pickUtf16EscapePair,
} from "../djson_utils.ts";
import { Random } from "../random.ts";
import { SequencePicker } from "../value_writer.ts";
import { flatten, weighted } from "../weight_tree.ts";
import { WeightedChoices } from "../weighted_choices.ts";
import type { Recipe } from "./mod.ts";

const utf16EscapePickers = new WeightedChoices<SequencePicker>(
    flatten([
        weighted(16, pickUtf16Escape()),
        weighted(1, pickUtf16EscapePair()),
    ]),
);

export const recipe: Recipe = {
    root: escapeStreamWriter(
        (rng: Random) => rng.choose(utf16EscapePickers)(rng),
        1024,
    ),
    seed: 49n,
    minBytes: Infinity,
    lineWidth: 100,
    maxDepth: 0,
};
