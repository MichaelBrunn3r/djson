import { assert, assertAlmostEquals } from "jsr:@std/assert";

/**
 * Sampler over a fixed set of weighted choices.
 * The `prob` values of the supplied choices should sum to approximately 1.
 */
export class WeightedChoices<T> {
  readonly #choices: readonly T[];
  readonly #cumProbs: Float64Array;

  constructor(choices: readonly Choice<T>[]) {
    assert(choices.length > 0, "weighted choices without choices");

    const picked: T[] = new Array(choices.length);
    const cumProbs = new Float64Array(choices.length);
    let cumProb = 0; // cumulative probability
    for (let i = 0; i < choices.length; i += 1) {
      const { value, prob } = choices[i];
      picked[i] = value;
      cumProb += prob;
      cumProbs[i] = cumProb;
    }

    assertAlmostEquals(
      cumProb,
      1,
      1e-6,
      "probabilities must sum to approximately 1",
    );

    this.#choices = picked;
    this.#cumProbs = cumProbs;
  }

  /** Maps a number in [0, 1) to a choice, weighted by each choices `prob`. */
  choose(num: number): T {
    for (let i = 0; i < this.#cumProbs.length; i += 1) {
      if (num < this.#cumProbs[i]) return this.#choices[i];
    }
    return this.#choices[this.#cumProbs.length - 1];
  }

  get length(): number {
    return this.#choices.length;
  }
}

export interface Choice<T> {
  value: T;
  /** Probability that this choice can be picked. */
  prob: number;
}
