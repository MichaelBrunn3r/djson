import type { BigRange, Range } from "./range.ts";
import type { WeightedChoices } from "./weighted_choices.ts";

/** 2 ** -32, maps a u32 onto [0, 1). */
const U32_SCALE = 2.3283064365386963e-10;

/** 2 ** 64, the width of the pool `intBig` draws from. */
const BIGINT_POOL_SIZE = 1n << 64n;

/** Helpers for drawing ranges, elements and weighted choices from any `Prng`. */
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

  /** Uniform integer in `[bounds.min, bounds.max]`. */
  int(bounds: Range): number {
    const offset = bounds.size <= 0x80000000
      ? (this.#prng.nextU32() * (U32_SCALE * bounds.size) | 0)
      : Math.floor(
        this.#prng.nextU32() * (U32_SCALE * bounds.size),
      );
    return bounds.min + offset;
  }

  /** Uniform integer in `[bounds.min, bounds.max]`. */
  intBig(bounds: BigRange): bigint {
    const span = bounds.size;
    if (span > BIGINT_POOL_SIZE) {
      throw new Error(
        `bounds too wide to draw from one pool, got size: ${span}`,
      );
    }

    const limit = BIGINT_POOL_SIZE - BIGINT_POOL_SIZE % span;
    let drawn;
    do {
      drawn = (BigInt(this.#prng.nextU32()) << 32n) |
        BigInt(this.#prng.nextU32());
    } while (drawn >= limit);
    return bounds.min + drawn % span;
  }

  /** Uniform float in `[bounds.min, bounds.max)`. */
  float(bounds: Range): number {
    return bounds.min +
      this.#prng.nextU32() * (U32_SCALE * (bounds.max - bounds.min));
  }

  /** Uniformly picks one element of `items`. */
  pick<T>(items: readonly T[]): T {
    return items[this.#prng.nextU32() * (U32_SCALE * items.length) | 0];
  }

  /** Picks from `choices` using a uniform number in `[0, 1)`. */
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
