export type TasksPins = Record<string, Pins>;
export type Pins = Record<string, string>;

export const HASHES_FILE = "scripts/hashes.json";
const HASHES_URL = new URL("hashes.json", import.meta.url);

export async function load_hashes(): Promise<TasksPins> {
    try {
        return JSON.parse(await Deno.readTextFile(HASHES_URL));
    } catch (error) {
        if (error instanceof Deno.errors.NotFound) {
            return {};
        }
        throw error;
    }
}

export async function save_hashes(hashes: TasksPins): Promise<void> {
    const sorted = Object.fromEntries(
        Object.entries(hashes)
            .sort(([left], [right]) => left.localeCompare(right))
            .map(([task, pins]) => [
                task,
                Object.fromEntries(
                    Object.entries(pins).sort(([left], [right]) =>
                        left.localeCompare(right)
                    ),
                ),
            ]),
    );
    await Deno.writeTextFile(
        HASHES_URL,
        `${JSON.stringify(sorted, null, 2)}\n`,
    );
}
