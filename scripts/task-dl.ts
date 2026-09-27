import { ensureFileSync } from "jsr:@std/fs/ensure-file";
import { existsSync } from "jsr:@std/fs/exists";
import { hashFile } from "./bench-data/utils.ts";
import { HASHES_FILE, load_hashes, save_hashes } from "./hashes.ts";

export const TASK_ID_DOWNLOAD = "dl";

interface ResourceDecl {
    /** Absolute URL the resource is downloaded from. */
    readonly url: string;
    /** Destination path, relative to the repository root. */
    readonly path: string;
}

export const RESOURCES: readonly ResourceDecl[] = [
    {
        url: "https://raw.githubusercontent.com/serde-rs/json-benchmark/refs/heads/master/data/citm_catalog.json",
        path: "pkgs/lib/benches/res/citm_catalog.json",
    },
];

export interface ArgsDownloadOptions {
    /** Record new hashes in the pin file instead of failing on a mismatch. */
    readonly update: boolean;
    /** Download every resource again, as if nothing had been downloaded yet. */
    readonly reinit: boolean;
}

/** Downloads every resource to its destination, pinning each URL in `scripts/hashes.json`. */
export async function cmd_download_resources(
    args: ArgsDownloadOptions,
): Promise<void> {
    const store = await load_hashes();
    const pinned = store[TASK_ID_DOWNLOAD] ?? {};
    const hashes: Record<string, string> = {};
    let changed = false;

    for (const resource of RESOURCES) {
        const expected = pinned[resource.url];

        // Check if a resource with the pinned hash already exists
        if (
            !args.reinit &&
            expected !== undefined &&
            existsSync(resource.path) &&
            (await hashFile(resource.path)) === expected
        ) {
            console.log(`✅ ${resource.url}`);
            continue;
        }

        const tmp = await download_resource(resource);
        try {
            verify_hash(
                resource.url,
                resource.path,
                expected,
                tmp.hash,
                args.update,
            );

            const updated = expected !== tmp.hash;
            Deno.renameSync(tmp.path, resource.path);
            console.log(`downloaded ${resource.url}`);
            console.log(`  -> ./${resource.path}`);
            console.log(`  SHA-256: ${tmp.hash}`);
            console.log(`  hash ${updated ? "updated" : "unchanged"}`);
            console.log(`  ${tmp.bytes} bytes`);
            changed ||= updated;
        } finally {
            try {
                Deno.removeSync(tmp.path);
            } catch {
                // Already renamed into place.
            }
        }

        hashes[resource.url] = tmp.hash;
    }

    if (changed) {
        store[TASK_ID_DOWNLOAD] = hashes;
        await save_hashes(store);
    }
}

/** A resource downloaded next to its destination, not yet in place. */
interface TmpResource {
    /** Path of the temporary resource. */
    readonly path: string;
    /** SHA-256 of the temporary resource. */
    readonly hash: string;
    /** Size of the resource, in bytes. */
    readonly bytes: number;
}

/** Downloads `resource` to a temporary path beside its destination. */
async function download_resource(
    resource: ResourceDecl,
): Promise<TmpResource> {
    const temporary = `${resource.path}.tmp`;
    ensureFileSync(temporary);

    const response = await fetch(resource.url);
    if (!response.ok) {
        throw new Error(
            `GET ${resource.url} failed: ${response.status} ${response.statusText}`,
        );
    }

    const bytes = new Uint8Array(await response.arrayBuffer());
    await Deno.writeFile(temporary, bytes);

    return {
        path: temporary,
        hash: await hashFile(temporary),
        bytes: bytes.length,
    };
}

function verify_hash(
    url: string,
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
            `no hash pinned for ${url} in ./${HASHES_FILE}\n` +
                `  downloaded SHA-256: ${actual}\n` +
                `  destination: ./${path}\n` +
                "  run again with --update to pin it",
        );
    }

    throw new Error(
        `hash verification failed for ${url}\n` +
            `  expected SHA-256: ${expected}\n` +
            `  actual SHA-256:   ${actual}\n` +
            `  destination: ./${path}\n` +
            "  the downloaded resource was discarded; check the URL " +
            "or run again with --update to accept the new hash",
    );
}
