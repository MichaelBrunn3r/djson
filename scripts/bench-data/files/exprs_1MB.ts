import {
    floatMethodWriters,
    intMethodWriters,
    nestingStringMethodWriters,
} from "../method_writers.ts";
import { anyValueProvider, type Catalogue, typeProvider } from "../provider.ts";
import { range } from "../range.ts";
import {
    boolWriter,
    delimitedWriter,
    DOUBLE_QUOTE,
    exprWriter,
    floatWriter,
    intWriter,
    listWriter,
    mapWriter,
    NO_DELIMITER,
    nullWriter,
    randomWordWriter,
    SINGLE_QUOTE,
    uniqueWordWriter,
    type ValueWriter,
} from "../value_writer.ts";
import { words } from "../words.ts";
import { weighted } from "../weight_tree.ts";
import { createCatalogue, mergeWeights, type WriterWeights } from "../utils.ts";
import type { Recipe } from "./mod.ts";

// Keep operands is safe bounds to avoid overflows
const OPERAND_BOUNDS = range(-50_000, 50_000);

/** Values the runtime still has to compute, unlike a static document. */
export const recipe: Recipe = {
    root: createRoot(),
    seed: 43n,
    minBytes: 1000 * 1024,
    lineWidth: 100,
    maxDepth: 32,
};

/** The document's single top-level value. */
function createRoot(): ValueWriter {
    const keys = uniqueWordWriter(words);
    const shallow = createCatalogue((catalogue) =>
        shallowWeights(catalogue, keys)
    );
    const operands = createCatalogue(operandWeights);
    const nesting = createCatalogue((catalogue) =>
        nestingWeights(catalogue, shallow, keys, operands)
    );
    return mapWriter(
        typeProvider("key", nesting),
        anyValueProvider(nesting),
    );
}

function nestingWeights(
    catalogue: Catalogue,
    shallow: Catalogue,
    mapKeys: ValueWriter,
    operands: Catalogue,
): WriterWeights {
    return mergeWeights(shallowWeights(shallow, mapKeys), {
        "int": [
            exprWriter(typeProvider("int", operands), range(2, 4)),
            weighted(1 / 3, intMethodWriters(catalogue, shallow)),
        ],
        "float": [
            exprWriter(typeProvider("float", operands), range(2, 4)),
            weighted(1 / 3, floatMethodWriters(catalogue, shallow)),
        ],
        "string": [
            weighted(1 / 3, nestingStringMethodWriters(catalogue, shallow)),
        ],
        "list": [
            listWriter(anyValueProvider(catalogue, shallow), range(0, 20)),
        ],
        "map": [
            mapWriter(
                typeProvider("key", catalogue, shallow),
                anyValueProvider(catalogue, shallow),
                { lenBounds: range(0, 20), firstMapId: 1 },
            ),
        ],
    });
}

/** Writers that keep to values needing no runtime. */
function shallowWeights(self: Catalogue, mapKeys: ValueWriter): WriterWeights {
    return {
        "int": [
            weighted(3, intWriter(range(-100, 100))),
            intWriter(range(-10_000_000, 10_000_000)),
        ],
        "float": [
            weighted(3, floatWriter(range(-100, 100), range(1, 8))),
            floatWriter(range(-10_000_000.0, 10_000_000.0), range(1, 8)),
        ],
        "bool": [boolWriter()],
        "string": [
            delimitedWriter(
                randomWordWriter(words, { countBounds: range(1, 6) }),
                [SINGLE_QUOTE, DOUBLE_QUOTE],
            ),
        ],
        "null": [nullWriter()],
        "list": [listWriter(anyValueProvider(self), range(0, 0))],
        "map": [
            mapWriter(
                typeProvider("key", self),
                anyValueProvider(self),
                { lenBounds: range(0, 0) },
            ),
        ],
        "key": [
            delimitedWriter(mapKeys, [
                NO_DELIMITER,
                SINGLE_QUOTE,
                DOUBLE_QUOTE,
            ]),
        ],
    };
}

function operandWeights(): WriterWeights {
    return {
        "int": [
            weighted(3, intWriter(range(-100, 100))),
            intWriter(OPERAND_BOUNDS),
        ],
        "float": [
            weighted(3, floatWriter(range(-100, 100), range(1, 8))),
            floatWriter(OPERAND_BOUNDS, range(1, 8)),
        ],
    };
}
