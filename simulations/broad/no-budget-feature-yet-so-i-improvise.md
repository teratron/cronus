---
id       = "no-budget-feature-yet-so-i-improvise"
tier     = "broad"
surfaces = ["cli"]
covers   = ["budget.show", "budget.set", "memory.store", "memory.search"]
roles    = ["owner"]
vantage  = "builtin-help"
world    = "fresh"
bound    = { steps = 30, wall_secs = 400, spend_usd = 1.50 }

[[perturbation]]
class = "abandonment"
at    = "gives up on the built-in budget commands entirely and falls back to tracking the number as a plain memory note instead"

[[obligation]]
id         = "an-unavailable-feature-says-so-plainly"
statement  = "attempting to set or show a budget in a workspace with an unconfigured budget store reports, in ordinary prose, that the feature is unavailable here"
decided_by = "observation-of-output"
evidence   = "both budget set and budget show report a clear, human-readable reason on their very first invocation in a fresh workspace, rather than a stack trace, a bare non-zero exit, or a number that looks real but was never set"
positive_control = "replays/budget-unavailable-reported-as-a-stack-trace.toml"

[[obligation]]
id         = "the-refusal-still-leaves-a-fallback-open"
statement  = "being told the budget feature is unavailable still leaves the actor free to record the same number some other way through this product"
decided_by = "observation-of-output"
evidence   = "after budget's refusal, storing the intended limit as an ordinary memory entry succeeds and that entry is later found by search"
---

## Persona

Someone trying to keep a personal monthly spending limit in view — rent, groceries, the
usual — who reaches for whatever this tool calls its budget feature, purely because the
top-level `--help` lists one.

## Goal

Set a monthly limit and check it later. If the dedicated feature turns out not to actually
work yet, fall back — the way anyone would — to just writing the number down somewhere the
tool will still let them find again, rather than giving up on the product entirely over one
missing piece.

## Notes

A discovery here is less about whether budget works today — it may honestly not — and more
about whether that absence is disclosed cleanly the first time someone reaches for it, and
whether the rest of the product still gives them a reasonable way to get the same job done
without it. A confusing refusal here would cost more trust than the missing feature itself.
