import type { Choice } from "./weighted_choices.ts";

/**
 * A tree of alternatives that tracks how likely each one is relative
 * to its siblings: a node's weight is weighed against the nodes beside it, so
 * the tree holds no absolute chances until it is flattened.
 *
 * @example
 * ```ts
 * [1, 2, 3, [4, 5, 6]]; // 1, 2, 3 at 1/4; 4, 5, 6 at 1/12
 * ["a", weighted(2, "b"), "c"]; // b 1/2, a and c 1/4 each
 * ```
 */
export type WeightTree<T> = readonly Node<T>[];

/** A leaf value, a nested subtree, or a weighted node. */
export type Node<T> = T | readonly Node<T>[] | Weighted<T>;

/** A node with the weight it carries among its siblings. */
export interface Weighted<T> {
  readonly weight: number;
  readonly value: T | readonly Node<T>[];
}

/** Tags `value` with the `weight` it carries among its siblings. */
export function weighted<T>(
  weight: number,
  value: T | readonly Node<T>[],
): Weighted<T> {
  if (!Number.isFinite(weight) || weight < 0) {
    throw new Error(
      `weight must be a finite number at least 0, got: ${weight}`,
    );
  }
  return { weight, value };
}

/** Flattens `tree` into one entry per value, with the chance it is picked. */
export function flatten<T>(tree: WeightTree<T>): Choice<T>[] {
  const out: Choice<T>[] = [];
  distribute(1, tree, out);
  return out;
}

/**
 * Distributes `total` among `nodes` by weight, descending into subtrees.
 *
 * @param out The array being constructed, one entry per leaf, in DFS order.
 */
function distribute<T>(
  total: number,
  nodes: readonly Node<T>[],
  out: Choice<T>[],
): void {
  let weightSum = 0;
  for (const node of nodes) weightSum += isWeighted(node) ? node.weight : 1;
  if (!(weightSum > 0)) throw new Error("nodes without a positive weight");

  for (const node of nodes) {
    let weight;
    let value;
    if (isWeighted(node)) {
      weight = node.weight;
      value = node.value;
    } else {
      weight = 1;
      value = node;
    }

    const share = (total * weight) / weightSum; // This nodes share of total
    if (isSubtree(value)) {
      distribute(share, value, out);
    } else {
      out.push({ value, prob: share });
    }
  }
}

function isWeighted<T>(node: Node<T>): node is Weighted<T> {
  return typeof node === "object" && node !== null && "weight" in node;
}

function isSubtree<T>(
  value: T | readonly Node<T>[],
): value is readonly Node<T>[] {
  return Array.isArray(value);
}
