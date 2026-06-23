# Nodus — Extraction Procedure

Step-by-step guide for moving `crates/nodus` out of the monorepo and publishing
it as a standalone crate on crates.io.

## Prerequisites

Verify Phase 3 validation tasks are green before starting:

```
cargo test -p nodus           # all tests pass
cargo clippy -p nodus -- -D warnings   # zero lints
cargo doc --no-deps -p nodus  # zero warnings
```

## 1. Create the standalone repository

```sh
gh repo create teratron/nodus --public --description "Declarative workflow DSL and Rust runtime" --clone
cd nodus
```

## 2. Copy crate contents

From the monorepo root:

```sh
cp -r crates/nodus/src      path/to/nodus/src
cp    crates/nodus/Cargo.toml  path/to/nodus/Cargo.toml
cp    crates/nodus/README.md   path/to/nodus/README.md
cp -r crates/nodus/tests    path/to/nodus/tests

# CI workflow goes to the repository root's .github/
mkdir -p path/to/nodus/.github/workflows
cp crates/nodus/.github/workflows/ci.yml path/to/nodus/.github/workflows/ci.yml
```

Remove `EXTRACTION.md` from the standalone repo — it is a monorepo-internal document.

## 3. Initial commit

```sh
cd path/to/nodus
git add .
git commit -m "feat: initial standalone release (extracted from teratron/cronus)"
git push origin main
```

## 4. Tag the release

```sh
git tag -a v0.1.0 -m "nodus v0.1.0 — initial standalone release"
git push origin main --tags
```

## 5. Publish to crates.io

```sh
cargo publish --dry-run   # verify packaging first
cargo publish
```

Ensure you are logged in: `cargo login`.

## 6. Update the Cronus monorepo

In `Cargo.toml` (workspace root), add the published version as a workspace dependency
and remove the path reference:

```toml
# Before
nodus = { path = "crates/nodus" }

# After (once v0.1.0 is published)
nodus = "0.1.0"
```

Run the monorepo test suite to confirm nothing regressed:

```sh
cargo test
```

## 7. Archive this phase

Once the standalone repo is live and Cronus tests pass:

- Mark Phase 3 Done in `.design/nodus/TASKS.md`.
- Run `/magic.run nodus` for final phase closure and archival.
