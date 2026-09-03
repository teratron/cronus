import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { CANONICAL_TOKENS } from "./tokens";

// Resolve against this test's own location (the shared tier), not cwd: the token
// files are tier-local, while styles.css stays at the composition root one level up.
const here = dirname(fileURLToPath(import.meta.url));
const readTier = (rel: string) => readFileSync(join(here, rel), "utf8");
const readRoot = (rel: string) => readFileSync(join(here, "..", rel), "utf8");

describe("design token contract (DI-3)", () => {
  it("the safe fallback set defines every canonical token", () => {
    const css = readTier("tokens.css");
    for (const token of CANONICAL_TOKENS) {
      expect(css).toContain(`${token}:`);
    }
  });

  it("the default scheme defines every canonical colour/shadow token in both modes", () => {
    // Per-mode files carry the scheme-specific values (colour + elevation);
    // type/spacing/radius/motion are mode-independent and stay in the fallback.
    const themed = CANONICAL_TOKENS.filter(
      (t) =>
        t.startsWith("--surface") ||
        t.startsWith("--text-primary") ||
        t.startsWith("--text-secondary") ||
        t.startsWith("--text-muted") ||
        t.startsWith("--text-inverse") ||
        t.startsWith("--border") ||
        t.startsWith("--focus") ||
        t.startsWith("--accent") ||
        t.startsWith("--success") ||
        t.startsWith("--warning") ||
        t.startsWith("--danger") ||
        t.startsWith("--info") ||
        t.startsWith("--shadow"),
    );
    for (const file of [
      "schemes/default/tokens.light.css",
      "schemes/default/tokens.dark.css",
    ]) {
      const css = readTier(file);
      for (const token of themed) {
        expect(css, `${file} is missing ${token}`).toContain(`${token}:`);
      }
    }
  });

  it("the @theme block maps a Tailwind utility for every canonical colour token", () => {
    const css = readTier("tokens.css");
    const colourTokens = CANONICAL_TOKENS.filter(
      (t) =>
        t.startsWith("--surface") ||
        t.startsWith("--text-primary") ||
        t.startsWith("--text-secondary") ||
        t.startsWith("--text-muted") ||
        t.startsWith("--text-inverse") ||
        t.startsWith("--border") ||
        t.startsWith("--focus") ||
        t.startsWith("--accent") ||
        t.startsWith("--success") ||
        t.startsWith("--warning") ||
        t.startsWith("--danger") ||
        t.startsWith("--info"),
    );
    for (const token of colourTokens) {
      expect(css).toContain(`--color-${token.slice(2)}: var(${token})`);
    }
  });

  it("styles.css wires the token layer and the default scheme", () => {
    const css = readRoot("styles.css");
    expect(css).toContain('@import "tailwindcss"');
    expect(css).toContain('@import "./shared/tokens.css"');
    expect(css).toContain('@import "./shared/schemes/default/tokens.light.css"');
    expect(css).toContain('@import "./shared/schemes/default/tokens.dark.css"');
  });
});
