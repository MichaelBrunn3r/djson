import { assert } from "jsr:@std/assert";
import type { Catalogue } from "./provider.ts";
import type { ValueWriter } from "./value_writer.ts";
import { WeightedChoices } from "./weighted_choices.ts";
import { flatten, type WeightTree } from "./weight_tree.ts";
import { Random } from "./random.ts";
import { range } from "./range.ts";

export type WriterWeights = Record<string, WeightTree<ValueWriter>>;

export function createCatalogue(
  weights: (catalogue: Catalogue) => WriterWeights,
): Catalogue {
  const catalogue: Catalogue = {};
  for (const [name, tree] of Object.entries(weights(catalogue))) {
    catalogue[name] = new WeightedChoices(flatten(tree));
  }
  return catalogue;
}

export function mergeWeights(
  ...tables: readonly WriterWeights[]
): WriterWeights {
  const merged: WriterWeights = {};
  for (const table of tables) {
    for (const [type, tree] of Object.entries(table)) {
      merged[type] = [...(merged[type] ?? []), ...tree];
    }
  }
  return merged;
}

/** Multiplier for each accepted suffix, case-insensitive. */
const BYTE_SUFFIXES: Readonly<Record<string, number>> = {
  "": 1,
  "b": 1,
  "k": 1024,
  "m": 1024 ** 2,
  "g": 1024 ** 3,
  "t": 1024 ** 4,
  "kb": 1000,
  "mb": 1000 ** 2,
  "gb": 1000 ** 3,
  "tb": 1000 ** 4,
  "kib": 1024,
  "mib": 1024 ** 2,
  "gib": 1024 ** 3,
  "tib": 1024 ** 4,
};

const BYTE_BUDGET = /^(\d+(?:\.\d+)?)\s*([a-z]*)$/i;

/** SHA-256 of the file at `path`, lowercase hex. */
export async function hashFile(path: string): Promise<string> {
  const bytes = new Uint8Array(await Deno.readFile(path));
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return new Uint8Array(digest).toHex();
}

export function formatDuration(ms: number): string {
  return ms < 1000 ? `${ms.toFixed(1)} ms` : `${(ms / 1000).toFixed(2)} s`;
}

export function formatRate(bytes: number, ms: number): string {
  const seconds = ms / 1000;
  if (!(seconds > 0)) return "∞ MB/s";
  return `${(bytes / 1_000_000 / seconds).toFixed(1)} MB/s`;
}

/** Parses an amount of bytes, e.g. `"8192"`, `"8kb"`, `"1.5mb"`, or `"2MiB"`. */
export function parseBytes(text: string): number {
  const match = BYTE_BUDGET.exec(text.trim());
  assert(
    match !== null,
    `expected a byte amount like "512", "10kb", or "2MiB", got "${text}"`,
  );

  const [, number, suffix] = match;
  const multiplier = BYTE_SUFFIXES[suffix.toLowerCase()];
  assert(
    multiplier !== undefined,
    `unknown byte suffix "${suffix}", expected one of ${
      Object.keys(BYTE_SUFFIXES).map((s) => `"${s}"`).join(", ")
    }`,
  );

  const bytes = Math.round(Number(number) * multiplier);
  assert(bytes > 0, `expected a positive byte amount, got "${text}"`);
  return bytes;
}

/** Reorders an array with a fixed-seed Fisher-Yates shuffle. */
export function shuffled<T>(
  array: readonly T[],
  seed: number | bigint,
): readonly T[] {
  const rng = Random.seeded(seed);
  const out = [...array];

  for (let i = out.length - 1; i > 0; i--) {
    const j = rng.int(range(0, i));
    const swap = out[i];
    out[i] = out[j];
    out[j] = swap;
  }

  return out;
}
