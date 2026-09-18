import { parseArgs } from "jsr:@std/cli/parse-args";
import { cmd_generate_files } from "./bench-data/cli.ts";

interface TaskOptions {
    /** Redo work the task could otherwise skip, as if it had never run. */
    readonly reinit: boolean;
    /**
     * Accept new hashes for the artifacts the task produces, instead of
     * failing when one no longer matches.
     */
    readonly update: boolean;
}

/** One unit of setup work. */
interface Task {
    readonly description: string;
    run(options: TaskOptions): void | Promise<void>;
}

const TASKS: Readonly<Record<string, Task>> = {
    "bench-data": {
        description: "generate benchmark files",
        run: ({ reinit, update }) => cmd_generate_files({ update, reinit }),
    },
};

async function main(): Promise<void> {
    const { reinit, update } = parseArgs(Deno.args, {
        string: ["reinit"],
        boolean: ["update"],
    });

    if (reinit !== undefined && !(reinit in TASKS)) {
        throw new Error(
            `unknown task "${reinit}", expected one of ${
                Object.keys(TASKS).join(", ")
            }`,
        );
    }

    const tasks = Object.entries(TASKS);
    for (const [index, [name, task]] of tasks.entries()) {
        const forced = name === reinit;
        console.log(
            `[${index + 1}/${tasks.length}] ${task.description}${
                forced ? " (reinit)" : ""
            }`,
        );
        await task.run({ reinit: forced, update });
    }
}

try {
    await main();
} catch (error) {
    // Deno would otherwise print a stack trace that buries the message.
    console.error(`error: ${error instanceof Error ? error.message : error}`);
    Deno.exitCode = 1;
}
