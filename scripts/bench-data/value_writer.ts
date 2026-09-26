import { assert } from "jsr:@std/assert";
import type { ValueWriterProvider } from "./provider.ts";
import type { Random } from "./random.ts";
import { type BigRange, type Range, range } from "./range.ts";
import type { Writer } from "./writer.ts";

const encoder = new TextEncoder();

const BYTE_SPACE = 0x20; // " "
const BYTE_OPEN_PAREN = 0x28; // "("
const BYTE_CLOSE_PAREN = 0x29; // ")"
const BYTE_COMMA = 0x2c; // ","
const BYTE_OPEN_BRACKET = 0x5b; // "["
const BYTE_CLOSE_BRACKET = 0x5d; // "]"
const BYTE_OPEN_BRACE = 0x7b; // "{"
const BYTE_CLOSE_BRACE = 0x7d; // "}"
export const BYTE_SINGLE_QUOTE = 0x27; // "'"
export const BYTE_DOUBLE_QUOTE = 0x22; // '"'
export const BYTE_UNDERSCORE = 0x5f; // "_"

export const NO_DELIMITER = -1;

const COMMA_SPACE = encoder.encode(", ");
const COLON_SPACE = encoder.encode(": ");
const EMPTY_LIST = encoder.encode("[]");
const EMPTY_MAP = encoder.encode("{}");

export interface ValueWriter {
  /** Writes a value to `writer`. */
  write(
    rng: Random,
    writer: Writer,
    depthBudget: number,
    /**
     * `true` if the parent writer requires to child to write a self-contained value.
     * Useful if the parent is a postfix operation like `"child".parent()`
     */
    requireDelimited?: boolean,
    /**
     * A number that is unique for the caller, but not globally.
     * A writer may interpret it however it likes: s an index, an offset, a seed, or nothing at all.
     */
    localId?: number,
  ): void;
}

/** Wraps whatever `inner` writes in a randomly picked pair of delimiters. */
export function delimitedWriter(
  inner: ValueWriter,
  delimiters: readonly number[],
): ValueWriter {
  assert(delimiters.length > 0, "expected at least one delimiter");
  return {
    write(
      rng: Random,
      writer: Writer,
      depthBudget: number,
      requireDelimited?: boolean,
      localId?: number,
    ): void {
      const delim = rng.pick(delimiters);
      const delimited = delim !== NO_DELIMITER;
      if (delimited) writer.writeByte(delim);
      inner.write(
        rng,
        writer,
        depthBudget,
        delimited ? false : requireDelimited,
        localId,
      );
      if (delimited) writer.writeByte(delim);
    },
  };
}

export function intWriter(
  bounds: Range,
  {
    separators = [],
    separatorChance = 0.5,
  }: { separators?: readonly number[]; separatorChance?: number } = {},
): ValueWriter {
  return {
    write(
      rng: Random,
      writer: Writer,
      _depthBudget: number,
      requireDelimited?: boolean,
    ): void {
      const value = rng.int(bounds);
      const wrap = requireDelimited === true && value < 0;
      if (wrap) writer.writeByte(BYTE_OPEN_PAREN);
      if (
        separators.length === 0 ||
        rng.nextFloat() >= separatorChance
      ) {
        writer.writeInt(value);
      } else {
        const digits = Math.abs(value).toString();
        const separator = rng.pick(separators);
        if (value < 0) writer.writeByte(0x2d);
        const firstGroupLength = digits.length % 3 || 3;
        writer.writeAscii(digits.slice(0, firstGroupLength));
        for (let i = firstGroupLength; i < digits.length; i += 3) {
          writer.writeByte(separator);
          writer.writeAscii(digits.slice(i, i + 3));
        }
      }
      if (wrap) writer.writeByte(BYTE_CLOSE_PAREN);
    },
  };
}

export function bigIntWriter(bounds: BigRange): ValueWriter {
  return {
    write(rng: Random, writer: Writer, _depthBudget: number): void {
      writer.writeBigInt(rng.intBig(bounds));
    },
  };
}

const EXPR_OPERATORS: readonly Uint8Array[] = [" + ", " - ", " * "].map(
  (operator) => encoder.encode(operator),
);
export function exprWriter(
  operands: ValueWriterProvider,
  countBounds: Range,
): ValueWriter {
  return {
    write(
      rng: Random,
      writer: Writer,
      _depthBudget: number,
      requireDelimited?: boolean,
    ): void {
      if (!writer.hasLineBudget) {
        operands(rng, true).write(rng, writer, 0, requireDelimited);
        return;
      }

      const wrap = requireDelimited === true;
      if (wrap) writer.writeByte(BYTE_OPEN_PAREN);
      const count = rng.int(countBounds);
      for (let i = 0; i < count; i++) {
        if (i > 0) {
          if (!writer.hasLineBudget) break;
          writer.writeBytes(rng.pick(EXPR_OPERATORS));
        }
        operands(rng, true).write(rng, writer, 0, true);
      }
      if (wrap) writer.writeByte(BYTE_CLOSE_PAREN);
    },
  };
}

export function floatWriter(bounds: Range, decimalsBounds: Range): ValueWriter {
  return {
    write(
      rng: Random,
      writer: Writer,
      _depthBudget: number,
      requireDelimited?: boolean,
    ): void {
      const decimals = rng.int(decimalsBounds);
      const value = rng.float(bounds);
      const wrap = requireDelimited === true && value < 0;
      if (wrap) writer.writeByte(BYTE_OPEN_PAREN);
      writer.writeAscii(value.toFixed(decimals));
      if (wrap) writer.writeByte(BYTE_CLOSE_PAREN);
    },
  };
}

const BOOLS: readonly Uint8Array[] = [
  encoder.encode("true"),
  encoder.encode("false"),
];
export function boolWriter(): ValueWriter {
  return {
    write(rng: Random, writer: Writer, _depthBudget: number): void {
      writer.writeBytes(rng.pick(BOOLS));
    },
  };
}

const NULL = encoder.encode("null");
export function nullWriter(): ValueWriter {
  return {
    write(_rng: Random, writer: Writer, _depthBudget: number): void {
      writer.writeBytes(NULL);
    },
  };
}

/** Writes `null` with chance `nullChance` and `inner` otherwise. */
export function nullableWriter(
  inner: ValueWriter,
  { nullChance = 0.5 }: { nullChance?: number } = {},
): ValueWriter {
  assert(
    nullChance >= 0 && nullChance <= 1,
    `null chance must be in [0, 1], got: ${nullChance}`,
  );

  const null_ = nullWriter();
  return {
    write(rng: Random, writer: Writer, depthBudget: number): void {
      if (rng.nextFloat() < nullChance) {
        null_.write(rng, writer, depthBudget);
        return;
      }
      inner.write(rng, writer, depthBudget);
    },
  };
}

const LETTERS: readonly Uint8Array[] = [
  ..."abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
].map((letter) => encoder.encode(letter));

/** Writes a random run of letters. */
export function stringWriter(
  { lenBounds = range(1, 8) }: { lenBounds?: Range } = {},
): ValueWriter {
  assert(
    lenBounds.isBounded,
    `string length bounds must be bounded, got max: ${lenBounds.max}`,
  );

  const scratch = new Uint8Array(lenBounds.max);

  return {
    write(rng: Random, writer: Writer, _depthBudget: number): void {
      const length = rng.int(lenBounds);
      for (let i = 0; i < length; i++) {
        scratch[i] = rng.pick(LETTERS)[0];
      }
      writer.writeBytes(scratch.subarray(0, length));
    },
  };
}

/** Picks one byte sequence to write. */
export type SequencePicker = (rng: Random) => Uint8Array;

/**
 * Writes a sequence of bytes, picked from `inner` or by `pickSequence`.
 *
 * Each round picks from `pickSequence` with chance `sequenceChance` and
 * `inner` otherwise. With `atLeastOne` the final round is forced to pick a
 * sequence if none was picked before.
 */
export function alternationWriter(
  inner: ValueWriter,
  pickSequence: SequencePicker,
  {
    countBounds = range(1, 4),
    sequenceChance = 0.5,
    atLeastOne = false,
  }: {
    countBounds?: Range;
    sequenceChance?: number;
    /** `true` if at least one sequence must be written. */
    atLeastOne?: boolean;
  } = {},
): ValueWriter {
  assert(
    sequenceChance >= 0 && sequenceChance <= 1,
    `sequence chance must be in [0, 1], got: ${sequenceChance}`,
  );

  return {
    write(
      rng: Random,
      writer: Writer,
      depthBudget: number,
      requireDelimited?: boolean,
      localId?: number,
    ): void {
      const count = rng.int(countBounds);
      let sequencesWritten = 0;
      for (let round = 0; round < count; round++) {
        const last = round === count - 1;
        const mustSequence = atLeastOne && sequencesWritten === 0 && last;

        if (mustSequence || rng.nextFloat() < sequenceChance) {
          writer.writeBytes(pickSequence(rng));
          sequencesWritten++;
        } else {
          inner.write(rng, writer, depthBudget, requireDelimited, localId);
        }
      }
    },
  };
}

/** Writes a run of words sampled from the pre-encoded `words`. */
export function randomWordWriter(
  words: readonly Uint8Array[],
  {
    countBounds = range(1, 4),
    separator = BYTE_SPACE,
  }: { countBounds?: Range; separator?: number } = {},
): ValueWriter {
  return {
    write(rng: Random, writer: Writer, _depthBudget: number): void {
      const count = rng.int(countBounds);
      for (let i = 0; i < count; i++) {
        if (i > 0) {
          if (!writer.hasLineBudget) break;
          writer.writeByte(separator);
        }
        writer.writeBytes(rng.pick(words));
      }
    },
  };
}

/**
 * On each invocation, writes a word from `words` that the caller has not seen yet (unique).
 *
 * Uniqueness depends on `localId` (passed by the caller) being unique.
 * Words are picked modulo `words.length`. If `localId` does not stay within those bounds or
 * does not increase linearly, words will repeat.
 *
 * Uniqueness is only guaranteed per caller. Invocations of different callers do not influence each other.
 */
export function uniqueWordWriter(words: readonly Uint8Array[]): ValueWriter {
  assert(words.length > 0, "expected at least one word");

  const length = words.length;

  return {
    write(
      _rng: Random,
      writer: Writer,
      _depthBudget: number,
      _requireDelimited?: boolean,
      localId?: number,
    ): void {
      assert(localId !== undefined, "a unique word needs an id");
      writer.writeBytes(words[localId % length]);
    },
  };
}

/**
 * Writes a list of items.
 *
 * Without `lenBounds` the list runs until the byte budget is exhausted.
 */
export function listWriter(
  provider: ValueWriterProvider,
  lenBounds?: Range,
): ValueWriter {
  return {
    write(rng: Random, writer: Writer, depthBudget: number): void {
      if (depthBudget < 0) {
        writer.writeBytes(EMPTY_LIST);
        return;
      }

      const length = lenBounds === undefined ? Infinity : rng.int(lenBounds);
      writer.writeByte(BYTE_OPEN_BRACKET);
      writer.indent();
      let written = 0;
      while (written < length) {
        if (!writer.hasBudget) break;

        if (written > 0) writer.writeByte(BYTE_COMMA);
        writer.newline();

        const stopNesting = depthBudget <= 0 || !writer.hasLineBudget;
        provider(rng, stopNesting).write(
          rng,
          writer,
          depthBudget - 1,
        );
        written++;
      }
      writer.dedent();
      if (written > 0) writer.newline();
      writer.writeByte(BYTE_CLOSE_BRACKET);
    },
  };
}

export function methodWriter(
  name: string,
  receiver: ValueWriterProvider,
  args: readonly ValueWriterProvider[],
): ValueWriter {
  const open = encoder.encode(`.${name}(`);

  return {
    write(rng: Random, writer: Writer, depthBudget: number): void {
      const stopNesting = depthBudget <= 0 || !writer.hasLineBudget;

      receiver(rng, stopNesting).write(
        rng,
        writer,
        depthBudget - 1,
        true,
      );
      writer.writeBytes(open);
      for (const [index, arg] of args.entries()) {
        if (index > 0) writer.writeBytes(COMMA_SPACE);
        arg(rng, stopNesting).write(rng, writer, depthBudget);
      }
      writer.writeByte(BYTE_CLOSE_PAREN);
    },
  };
}

/** Writes a map of keys to values. */
export function mapWriter(
  keys: ValueWriterProvider,
  values: ValueWriterProvider,
  {
    /**
     * Min/max number of entries a map can write.
     * If omitted, the map writes entries until the byte budget is exhausted.
     */
    lenBounds,
    /**
     * The map id / tile this writer starts at.
     * Allows giving independent writers disjoint tile ranges.
     */
    firstMapId = 0,
  }: { lenBounds?: Range; firstMapId?: number } = {},
): ValueWriter {
  // Each map is placed in a tile that can fit the max number of map entries this writer can produce.
  // Subsequent non-empty maps are placed in subsequent tiles.
  const tileSize = (lenBounds?.max ?? 0) + 1;
  let nextMapId = firstMapId;

  return {
    write(rng: Random, writer: Writer, depthBudget: number): void {
      const mapId = nextMapId;

      if (depthBudget < 0) {
        writer.writeBytes(EMPTY_MAP);
        return;
      }

      const numEntries = lenBounds === undefined
        ? Infinity
        : rng.int(lenBounds);
      let numEntriesWritten = 0;

      // Start with '{'
      writer.writeByte(BYTE_OPEN_BRACE);
      writer.indent();

      while (numEntriesWritten < numEntries) {
        if (!writer.hasBudget) break;

        // Terminate current entry with ',\n'
        if (numEntriesWritten > 0) writer.writeByte(BYTE_COMMA);
        writer.newline();

        // Only advance the global map id if this map is not empty
        if (numEntriesWritten === 0) {
          nextMapId++;
        }

        const stopNesting = depthBudget <= 0 || !writer.hasLineBudget;
        keys(rng, stopNesting).write(
          rng,
          writer,
          depthBudget - 1,
          undefined,
          tileSize * mapId + numEntriesWritten,
        );
        writer.writeBytes(COLON_SPACE);
        values(rng, stopNesting).write(
          rng,
          writer,
          depthBudget - 1,
        );
        numEntriesWritten++;
      }

      // Finish with '}'
      writer.dedent();
      if (numEntriesWritten > 0) writer.newline();
      writer.writeByte(BYTE_CLOSE_BRACE);
    },
  };
}

const PATTERN_RUST_IDENTIFIER = /^[A-Za-z_][A-Za-z0-9_]*$/;

/** Writes a map that holds `fields`, in a shuffled order. */
export function structWriter(
  fields: Readonly<Record<string, readonly [ValueWriter, boolean]>>,
  { optionalChance = 0.5 }: { optionalChance?: number } = {},
): ValueWriter {
  assert(
    optionalChance >= 0 && optionalChance <= 1,
    `optional chance must be in [0, 1], got: ${optionalChance}`,
  );

  const members = Object.entries(fields).map(([name, [value, optional]]) => {
    assert(
      PATTERN_RUST_IDENTIFIER.test(name),
      `struct field name must be a Rust identifier, got: ${name}`,
    );
    return { name: encoder.encode(`"${name}": `), value, optional };
  });

  assert(members.length > 0, "a struct needs at least one field");

  // Write order of the members, reshuffled in place for every value.
  const order = members.map((_, index) => index);

  return {
    write(rng: Random, writer: Writer, depthBudget: number): void {
      // Fisher-Yates, back to front.
      for (let i = order.length - 1; i > 0; i--) {
        const j = rng.int(range(0, i));
        const swap = order[i];
        order[i] = order[j];
        order[j] = swap;
      }

      writer.writeByte(BYTE_OPEN_BRACE);
      writer.indent();
      let written = 0;
      for (const index of order) {
        const member = members[index];

        if (
          member.optional &&
          optionalChance > 0 &&
          rng.nextFloat() < optionalChance
        ) {
          continue;
        }

        if (written > 0) writer.writeByte(BYTE_COMMA);
        writer.newline();
        writer.writeBytes(member.name);
        member.value.write(rng, writer, depthBudget);
        written++;
      }
      writer.dedent();
      if (written > 0) writer.newline();
      writer.writeByte(BYTE_CLOSE_BRACE);
    },
  };
}
