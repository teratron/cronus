#!/usr/bin/env node
// Boundary guard: fails the build if cronus-domain gains a normal dependency
// outside its allowlist. The crate split enforces the tier model at compile
// time only if this check runs on every build — otherwise the layout drifts
// exactly as it did before the split, silently.

import { execFileSync } from "node:child_process";

const ALLOWED = new Set(["cronus-contract", "nodus", "blake3", "chrono", "cron"]);

const raw = execFileSync("cargo", ["metadata", "--format-version=1"], {
    maxBuffer: 1024 * 1024 * 64,
}).toString("utf8");
const metadata = JSON.parse(raw);

const domainPkg = metadata.packages.find((p) => p.name === "cronus-domain");
if (!domainPkg) {
    console.error("error: no package named cronus-domain in the workspace metadata");
    process.exit(1);
}

const namesById = new Map(metadata.packages.map((p) => [p.id, p.name]));
const node = metadata.resolve.nodes.find((n) => n.id === domainPkg.id);

const normalDeps = node.deps
    .filter((d) => d.dep_kinds.some((k) => k.kind === null))
    .map((d) => namesById.get(d.pkg));

const offenders = normalDeps.filter((name) => !ALLOWED.has(name));

if (offenders.length > 0) {
    console.error(`error: cronus-domain has forbidden normal dependencies: ${offenders.join(", ")}`);
    console.error(`allowlist: ${[...ALLOWED].join(", ")}`);
    process.exit(1);
}

console.log(`ok: cronus-domain's normal dependencies are within the allowlist (${[...ALLOWED].join(", ")})`);
