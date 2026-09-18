import { assert } from "jsr:@std/assert/assert";
import {
  Catalogue,
  countProvider,
  fixedProvider,
  typeProvider,
  ValueWriterProvider,
} from "./provider.ts";
import { Range, range } from "./range.ts";
import { floatWriter, intWriter, methodWriter } from "./value_writer.ts";
import type { ValueWriter } from "./value_writer.ts";

/** Every method in djson whose result is a string. */
export function nestingStringMethodWriters(
  nestingCatalogue: Catalogue,
  nonNestingCatalogue: Catalogue,
): ValueWriter[] {
  const string = typeProvider(
    "string",
    nestingCatalogue,
    nonNestingCatalogue,
  );
  const counts = countProvider(range(1, 10));

  return [
    // No args
    methodWriter("trim", string, []),
    methodWriter("trim_start", string, []),
    methodWriter("trim_end", string, []),
    methodWriter("uppercase", string, []),
    methodWriter("lowercase", string, []),
    methodWriter("reverse", string, []),

    // With args
    methodWriter("remove_prefix", string, [string]),
    methodWriter("remove_suffix", string, [string]),
    methodWriter("replace", string, [string, string]),
    methodWriter("repeat", string, [counts]),
    methodWriter("pad_start", string, [counts, string]),
    methodWriter("pad_end", string, [counts, string]),
  ];
}

/** Every method in djson whose result is an int. */
export function intMethodWriters(
  nestingCatalogue: Catalogue,
  nonNestingCatalogue: Catalogue,
): ValueWriter[] {
  const int = typeProvider("int", nestingCatalogue, nonNestingCatalogue);
  const float = typeProvider("float", nestingCatalogue, nonNestingCatalogue);
  const string = typeProvider(
    "string",
    nestingCatalogue,
    nonNestingCatalogue,
  );
  const list = typeProvider("list", nestingCatalogue, nonNestingCatalogue);
  const map = typeProvider("map", nestingCatalogue, nonNestingCatalogue);

  return [
    // Called on an int
    methodWriter("abs", int, []),
    methodWriter(
      "clamp",
      int,
      clampBoundsProvider(range(-100, 100), intWriter),
    ),
    methodWriter("max", int, [int]),
    methodWriter("min", int, [int]),

    // Called on a float
    methodWriter("ceil", float, []),
    methodWriter("floor", float, []),
    methodWriter("round", float, []),

    // Called on a string
    methodWriter("len_chars", string, []),
    methodWriter("len_bytes", string, []),
    methodWriter("count", string, [string]),
    methodWriter("find", string, [string]),

    // Called on a list or a map
    methodWriter("len", list, []),
    methodWriter("len", map, []),
  ];
}

/** Every method in djson whose result is a float. */
export function floatMethodWriters(
  nestingCatalogue: Catalogue,
  nonNestingCatalogue: Catalogue,
): ValueWriter[] {
  const float = typeProvider("float", nestingCatalogue, nonNestingCatalogue);
  const positiveInt = fixedProvider(intWriter(range(1, 100)));
  const positiveFloat = fixedProvider(floatWriter(range(1, 100), range(1, 8)));

  return [
    // Called on an int
    methodWriter("ln", positiveInt, []),
    methodWriter("log", positiveInt, [fixedProvider(intWriter(range(2, 10)))]),
    methodWriter("log2", positiveInt, []),
    methodWriter("log10", positiveInt, []),
    methodWriter("sqrt", positiveInt, []),

    // Called on a float
    methodWriter("abs", float, []),
    methodWriter(
      "clamp",
      float,
      clampBoundsProvider(
        range(-100.0, 100.0),
        (bounds) => floatWriter(bounds, range(1, 8)),
      ),
    ),
    methodWriter("ln", positiveFloat, []),
    methodWriter("log", positiveFloat, [
      fixedProvider(floatWriter(range(2, 10), range(1, 8))),
    ]),
    methodWriter("log2", positiveFloat, []),
    methodWriter("log10", positiveFloat, []),
    methodWriter("max", float, [float]),
    methodWriter("min", float, [float]),
    methodWriter("sqrt", positiveFloat, []),
  ];
}

export function clampBoundsProvider(
  bounds: Range,
  writer: (bounds: Range) => ValueWriter,
): readonly [ValueWriterProvider, ValueWriterProvider] {
  assert(bounds.min < 0 && bounds.max > 0);
  return [
    () => writer(range(bounds.min, 0)),
    () => writer(range(0, bounds.max)),
  ];
}
