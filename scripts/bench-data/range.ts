export class Range {
  readonly min: number;
  readonly max: number;

  constructor(min: number, max: number) {
    this.min = min;
    this.max = max;
  }

  /** Yields each number from `min` to `max`, inclusive. */
  *[Symbol.iterator](): Generator<number> {
    for (let n = this.min; n <= this.max; n++) yield n;
  }

  /** How many values the range covers. */
  get size(): number {
    return this.max - this.min + 1;
  }

  get isBounded(): boolean {
    return this.max < Number.MAX_SAFE_INTEGER;
  }
}

/** Creates an inclusive range from `min` to `max`. */
export function range(min: number, max: number): Range {
  return new Range(min, max);
}

export function unbounded(min: number): Range {
  return new Range(min, Number.MAX_SAFE_INTEGER);
}

/** A range of bigints (slow) */
export class BigRange {
  readonly min: bigint;
  readonly max: bigint;

  constructor(min: bigint, max: bigint) {
    this.min = min;
    this.max = max;
  }

  get size(): bigint {
    return this.max - this.min + 1n;
  }
}

export function intRange(min: bigint, max: bigint): BigRange {
  return new BigRange(min, max);
}
