import type { Range } from "./range.ts";
import type { WeightedChoices } from "./weighted_choices.ts";

/** 2 ** -32, maps a u32 onto [0, 1). */
const U32_SCALE = 2.3283064365386963e-10;

/**
 * Helpers for drawing shaped values (ranges, elements, weighted choices) from
 * any `Prng`.
 *
 * Every draw is derived from a raw u32, so nothing here allocates. Each method
 * documents why it is written the way it is.
 */
export class Random {
  readonly #prng: RNG;

  constructor(prng: RNG) {
    this.#prng = prng;
  }

  /** Creates a `Random` over the default algorithm from a deterministic seed. */
  static seeded(seed: bigint | number): Random {
    return new Random(new Sfc32(seed));
  }

  /** The next raw u32 in `[0, 2 ** 32)`. */
  nextU32(): number {
    return this.#prng.nextU32();
  }

  /** The next float in `[0, 1)`. */
  nextFloat(): number {
    return this.#prng.nextU32() * U32_SCALE;
  }

  /**
   * Uniform integer in `bounds`, inclusive of both ends.
   *
   * `(u * 2**-32 * size) | 0` beats the obvious alternatives: `prng_bench.ts`
   * measured it ~20% faster than `Math.floor(u * 2**-32 * size)` and more than
   * twice as fast as `u % size`, which is also biased towards low residues.
   *
   * Truncating is exact, not merely cheap, for every `size` up to
   * `Number.MAX_SAFE_INTEGER`: the product stays below that bound, so no
   * precision is lost, and both operands are non-negative, so `| 0` agrees
   * with `Math.floor`.
   */
  int(bounds: Range): number {
    return bounds.min + (this.#prng.nextU32() * (U32_SCALE * bounds.size) | 0);
  }

  /**
   * Uniform float in `bounds`, half-open at the top.
   *
   * Derived from the raw u32 rather than from `nextFloat`, so the whole scale
   * `2**-32 * span` is one multiply instead of two.
   */
  float(bounds: Range): number {
    return bounds.min +
      this.#prng.nextU32() * (U32_SCALE * (bounds.max - bounds.min));
  }

  /**
   * Uniformly picks one element of `items`.
   *
   * Indexes straight off the u32 instead of going through
   * `int(range(0, items.length - 1))`, which would allocate a `Range` on every
   * call.
   */
  pick<T>(items: readonly T[]): T {
    return items[this.#prng.nextU32() * (U32_SCALE * items.length) | 0];
  }

  /**
   * Picks from `choices` using a uniform number in [0, 1).
   *
   * `WeightedChoices` walks cumulative probabilities, so it wants the fraction
   * `nextFloat` already returns rather than a scaled index.
   */
  choose<T>(choices: WeightedChoices<T>): T {
    return choices.choose(this.nextFloat());
  }
}

// region Algorithms

/** A source of random 32 bit words. */
export interface RNG {
  /** The next raw u32 in `[0, 2 ** 32)`. */
  nextU32(): number;
}

export class Sfc32 implements RNG {
  #a: number;
  #b: number;
  #c: number;
  #d: number;

  constructor(seed: bigint | number = 1) {
    const nextWord = splitmix32(seed);
    this.#a = nextWord();
    this.#b = nextWord();
    this.#c = nextWord();
    this.#d = nextWord();
  }

  nextU32(): number {
    const t = (this.#a + this.#b) | 0;
    this.#a = this.#b ^ (this.#b >>> 9);
    this.#b = (this.#c + (this.#c << 3)) | 0;
    this.#c = (this.#c << 21) | (this.#c >>> 11);
    this.#d = (this.#d + 1) | 0;
    const r = (t + this.#d) | 0;
    this.#c = (this.#c + r) | 0;
    return r >>> 0;
  }
}

export function splitmix32(seed: bigint | number): () => number {
  let z = Number(BigInt.asIntN(32, BigInt(seed))) | 0;
  return () => {
    z = (z + 0x9e3779b9) | 0;
    let t = z ^ (z >>> 16);
    t = Math.imul(t, 0x21f0aaad);
    t ^= t >>> 15;
    t = Math.imul(t, 0x735a2d97);
    return (t ^ (t >>> 15)) >>> 0;
  };
}
// endregion Algorithms
