import { assert } from "jsr:@std/assert";
import { ensureFileSync } from "jsr:@std/fs/ensure-file";
import { existsSync } from "jsr:@std/fs/exists";
import {
  BENCHMARK_FILES,
  type BenchmarkFile,
  HASHES_FILE,
  load_file_hashes,
  type Recipe,
  save_file_hashes,
} from "./files/mod.ts";
import { Random } from "./random.ts";
import {
  MAX_INDENT_CACHE_SIZE,
  Writer,
  type WriterResource,
} from "./writer.ts";
import { formatDuration, formatRate, hashFile } from "./utils.ts";

export interface ArgsStdout {
  /** Seed of the value stream; the caller resolves any default. */
  readonly seed: bigint;
  /** Least number of bytes to write. */
  readonly minBytes: number;
  /** Target line width. */
  readonly lineWidth: number;
  /** How deep values may nest. */
  readonly maxDepth: number;
}

/** Writes one document to standard output. */
export function cmd_write_stdout(options: ArgsStdout): void {
  write_document(
    Deno.stdout,
    {
      ...BENCHMARK_FILES.exprs_1MB.recipe,
      minBytes: options.minBytes,
      lineWidth: options.lineWidth,
      maxDepth: options.maxDepth,
    },
    options.seed,
  );
}

export interface ArgsGenerateOptions {
  /** Record new hashes in the pin file instead of failing on a mismatch. */
  readonly update: boolean;
  /** Regenerate every document, as if nothing had been generated yet. */
  readonly reinit: boolean;
}

/** Writes every document to its destination, checking each against its pin in `scripts/hashes.json`. */
export async function cmd_generate_files(
  args: ArgsGenerateOptions,
): Promise<void> {
  const pinned = await load_file_hashes();
  const hashes: Record<string, string> = {};
  let changed = false;

  for (const [name, file] of Object.entries(BENCHMARK_FILES)) {
    const expected = pinned[file.path];

    // Check if a document with the expected hash already exists
    if (
      !args.reinit &&
      expected !== undefined &&
      existsSync(file.path) &&
      (await hashFile(file.path)) === expected
    ) {
      console.log(`✅ ${name} ./${file.path}`);
      hashes[file.path] = expected;
      continue;
    }

    const generated = await write_temporary(file);
    try {
      verify_hash(file.path, expected, generated.hash, args.update);

      const updated = expected !== generated.hash;
      Deno.renameSync(generated.temporary, file.path);
      console.log(`generated ${name} ./${file.path}`);
      console.log(`  SHA-256: ${generated.hash}`);
      console.log(`  hash ${updated ? "updated" : "unchanged"}`);
      console.log(
        `  ${generated.bytes} bytes in ` +
          `${formatDuration(generated.elapsedMs)} ` +
          `(${formatRate(generated.bytes, generated.elapsedMs)})`,
      );
      changed ||= updated;
    } finally {
      try {
        Deno.removeSync(generated.temporary);
      } catch {
        // Already renamed into place.
      }
    }

    hashes[file.path] = generated.hash;
  }

  if (changed) {
    await save_file_hashes(hashes);
  }
}

/** A document written next to its destination, not yet in place. */
interface Generated {
  /** Path of the temporary document. */
  readonly temporary: string;
  /** SHA-256 of the temporary document. */
  readonly hash: string;
  /** Size of the document, in bytes. */
  readonly bytes: number;
  /** Time the generation took, in milliseconds. */
  readonly elapsedMs: number;
}

/** Writes `file`'s document to a temporary path beside its destination. */
async function write_temporary(file: BenchmarkFile): Promise<Generated> {
  const temporary = `${file.path}.tmp`;
  ensureFileSync(temporary);

  const started = performance.now();
  write_document(
    Deno.openSync(temporary, {
      create: true,
      truncate: true,
      write: true,
    }),
    file.recipe,
  );
  const elapsedMs = performance.now() - started;

  return {
    temporary,
    hash: await hashFile(temporary),
    bytes: Deno.statSync(temporary).size,
    elapsedMs,
  };
}

function verify_hash(
  path: string,
  expected: string | undefined,
  actual: string,
  update: boolean,
): void {
  if (update || expected === actual) {
    return;
  }

  if (expected === undefined) {
    throw new Error(
      `no hash pinned for ./${path} in ./${HASHES_FILE}\n` +
        `  generated SHA-256: ${actual}\n` +
        "  run again with --update to pin it",
    );
  }

  throw new Error(
    `hash verification failed for ./${path}\n` +
      `  expected SHA-256: ${expected}\n` +
      `  actual SHA-256:   ${actual}\n` +
      "  the generated document was discarded; check the generator " +
      "inputs or run again with --update to accept the new hash",
  );
}

export function write_document(
  target: WriterResource,
  recipe: Recipe,
  seed: bigint = recipe.seed,
): void {
  assert(
    recipe.maxDepth + 1 <= MAX_INDENT_CACHE_SIZE,
    `recipe nesting is > ${MAX_INDENT_CACHE_SIZE}`,
  );

  const writer = new Writer(
    target,
    "  ",
    recipe.lineWidth,
    recipe.minBytes,
  );
  try {
    recipe.root.write(Random.seeded(seed), writer, recipe.maxDepth);
    writer.newline();
  } finally {
    writer.close();
  }
}
