import { parseArgs } from "jsr:@std/cli/parse-args";
import { assert } from "jsr:@std/assert";
import {
  type ArgsGenerateOptions,
  type ArgsStdout,
  cmd_generate_files,
  cmd_write_stdout,
} from "./cli.ts";
import { parseBytes } from "./utils.ts";

const DEFAULT_MAX_DEPTH = 32;

const USAGE = `usage: main.ts <command> [options]

commands:
  stdout    write one document to standard output
  generate  write every document to its destination

generate options:
  --update      record new hashes in scripts/hashes.json

stdout options:
  --seed, -s    seed of the random source (default: a random seed)
  --budget, -b  least amount of bytes to write (default "2kb")
  --width, -w   target line width (default 100)
  --depth, -d   maximum nesting depth (default ${DEFAULT_MAX_DEPTH})`;

/** Runs the command named by the first argument. */
async function main(): Promise<void> {
  const [command, ...args] = Deno.args;
  switch (command) {
    case "stdout":
      return cmd_write_stdout(parse_args_stdout(args));
    case "generate":
      return await cmd_generate_files(parse_args_generate(args));
    default:
      console.log(USAGE);
      if (command !== undefined) {
        throw new Error(`unknown command "${command}"`);
      }
  }
}

function parse_args_stdout(args: readonly string[]): ArgsStdout {
  const { seed, budget, width, depth } = parseArgs(args, {
    string: ["seed", "budget", "width", "depth"],
    alias: { s: "seed", b: "budget", w: "width", d: "depth" },
  });

  const maxDepth = depth === undefined ? DEFAULT_MAX_DEPTH : Number(depth);
  assert(
    Number.isSafeInteger(maxDepth) && maxDepth >= 0,
    `expected a non-negative nesting depth, got "${depth}"`,
  );

  // Parse --seed or use a random one and report it
  const rngSeed = seed === undefined ? randomSeed() : BigInt(seed);
  if (seed === undefined) {
    console.error(`# seed ${rngSeed}`);
  }

  return {
    seed: rngSeed,
    minBytes: parseBytes(budget ?? "2kb"),
    lineWidth: parseBytes(width ?? "100"),
    maxDepth,
  };
}

function parse_args_generate(args: readonly string[]): ArgsGenerateOptions {
  const { update } = parseArgs(args, { boolean: ["update"] });
  return { update, reinit: false };
}

function randomSeed(): bigint {
  const words = new BigUint64Array(1);
  crypto.getRandomValues(words);
  return words[0];
}

try {
  await main();
} catch (error) {
  // Avoid printing a stack trace
  console.error(`error: ${error instanceof Error ? error.message : error}`);
  Deno.exitCode = 1;
}
