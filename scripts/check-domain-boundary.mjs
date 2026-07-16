#!/usr/bin/env node
// Boundary guard: fails the build if cronus-domain gains a normal dependency
// outside its allowlist. The crate split enforces the tier model at compile
// time only if this check runs on every build — otherwise the layout drifts
// exactly as it did before the split, silently.
//
// The adapter crates — cronus-store-local, cronus-auth-local, and
// cronus-model-local (the model-transport adapter) — are deliberately ABSENT
// from the allowlist: domain reaches them only through the contract traits
// (`UserDataStore`, `AuthProvider`, `InferenceBackend`, …), never by a direct
// dependency. A `cronus-domain -> cronus-model-local` edge is exactly the
// inward-dependency drift (INV-8) this guard exists to reject.
//
// Run `--self-test` to exercise the failure path itself: it asserts the real
// tree is clean AND that an injected `domain -> model-local` edge is flagged,
// so the guard's rejection logic is proven, not just assumed.

import { execFileSync } from "node:child_process";
import assert from "node:assert/strict";

const ALLOWED = new Set(["cronus-contract", "nodus", "blake3", "chrono", "cron"]);

// The adapter crates domain must never depend on directly (checked by the
// self-test to stay off the allowlist).
const FORBIDDEN_ADAPTERS = ["cronus-store-local", "cronus-auth-local", "cronus-model-local"];

// Pure core, separated from I/O so the self-test can drive it with synthetic
// inputs: given cronus-domain's normal-dependency names, return the ones
// outside the allowlist.
function findOffenders(depNames, allowed = ALLOWED) {
    return depNames.filter((name) => !allowed.has(name));
}

// Read cronus-domain's resolved normal (non-dev, non-build) dependencies from
// cargo metadata.
function domainNormalDeps() {
    const raw = execFileSync("cargo", ["metadata", "--format-version=1"], {
        maxBuffer: 1024 * 1024 * 64,
    }).toString("utf8");
    const metadata = JSON.parse(raw);

    const domainPkg = metadata.packages.find((p) => p.name === "cronus-domain");
    if (!domainPkg) {
        throw new Error("no package named cronus-domain in the workspace metadata");
    }
    const namesById = new Map(metadata.packages.map((p) => [p.id, p.name]));
    const node = metadata.resolve.nodes.find((n) => n.id === domainPkg.id);

    return node.deps
        .filter((d) => d.dep_kinds.some((k) => k.kind === null))
        .map((d) => namesById.get(d.pkg));
}

// Exercise the guard's own failure path so "the boundary is enforced" is a
// tested fact, not a claim.
function selfTest() {
    const realDeps = domainNormalDeps();

    // 1. The real tree is clean.
    assert.deepEqual(
        findOffenders(realDeps),
        [],
        `the real cronus-domain deps must be within the allowlist; offenders: ${findOffenders(realDeps).join(", ")}`,
    );

    // 2. An injected `domain -> cronus-model-local` edge is rejected — the
    //    exact drift this guard must catch (T-17D01).
    const injected = [...realDeps, "cronus-model-local"];
    assert.ok(
        findOffenders(injected).includes("cronus-model-local"),
        "an injected domain->model-local edge must be flagged as a boundary violation",
    );

    // 3. No adapter crate is ever on the allowlist (belt and suspenders — a
    //    future edit adding one would silently defeat the guard).
    for (const adapter of FORBIDDEN_ADAPTERS) {
        assert.ok(!ALLOWED.has(adapter), `${adapter} must not be on the domain allowlist`);
    }

    console.log(
        "ok: boundary-guard self-test passed (real tree clean; an injected domain->model-local edge is rejected)",
    );
}

function main() {
    if (process.argv.includes("--self-test")) {
        selfTest();
        return;
    }

    const offenders = findOffenders(domainNormalDeps());
    if (offenders.length > 0) {
        console.error(`error: cronus-domain has forbidden normal dependencies: ${offenders.join(", ")}`);
        console.error(`allowlist: ${[...ALLOWED].join(", ")}`);
        process.exit(1);
    }
    console.log(`ok: cronus-domain's normal dependencies are within the allowlist (${[...ALLOWED].join(", ")})`);
}

main();
