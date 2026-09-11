# Usage Simulation Scenarios

A scenario in this directory is a declarative description of **who wants what, and what they
are allowed to know** — never of which commands they type. An agent (or, later, a human)
drives the real product toward the stated goal by whatever route it finds; the route is the
thing a scenario deliberately does not fix, because the routes nobody thought to script are
exactly the ones worth watching.

## Directory layout

- `short/` — the cheap set: journeys whose breakage would make the product unusable. Runs
  often.
- `broad/` — one scenario per area of the product, with situation development included. Runs
  per milestone or release candidate.
- `exhaustive/` — every surface, every action, every role, the full set of situation
  developments below. Runs on demand.

Placement in a directory is not decorative — it sets how often that scenario actually runs.

## File format

A scenario file is Markdown with a `---`-delimited TOML block at the top, followed by prose.

```toml
---
id       = "a-short-stable-name"
tier     = "short"                 # short | broad | exhaustive
surfaces = ["cli"]                 # which shipped surfaces are in play
covers   = ["board.add", "board.list"]   # the product actions this scenario touches
roles    = ["owner"]                # which rights the actor has (today: always one value)
vantage  = "builtin-help"           # what the actor may consult — see below
world    = "fresh"                  # fresh | seeded-small | seeded-large | corrupted
bound    = { steps = 40, wall_secs = 600, spend_usd = 1.50 }

[[perturbation]]
class = "reversal"
at    = "when, in the story, this happens"

[[obligation]]
id         = "short-stable-name"
statement  = "an observable outcome, stated as a fact about the world after the run"
decided_by = "observation-of-output"      # observation-of-output | observation-of-state | judgement
evidence   = "exactly what must be shown for this to count as met"
# positive_control is required whenever the statement asserts something did NOT happen.
positive_control = "a pinned case where the thing being denied actually occurs"
---

## Persona

Who is acting, in plain terms, and why they care.

## Goal

What they want, in their own words — not in the product's command names.

## Notes

What a discovery on this scenario would most likely mean.
```

Unrecognized top-level keys, a missing `bound`, and an obligation set that could never fail a
run (empty, duplicate ids, no evidence, phrased as an activity instead of an outcome, or an
absence claim with no `positive_control`) are all refused when the file is loaded — never
silently skipped.

## Vantage: what the actor may know

Every judgement about whether something was *discoverable*, an error was *clear*, or a name
was *findable* is only meaningful relative to what the actor was allowed to consult. Pick
exactly one:

| Value | The actor may consult |
| --- | --- |
| `none` | nothing at all — pure trial and error |
| `builtin-help` | the product's own `--help` output, at each step, and nothing else |
| `published-docs` | the built-in help plus whatever documentation ships with the product |
| `prior-use` | everything above, plus the memory of having used this product before |

A scenario using `builtin-help` or below must not assume the actor read this file, the source
code, or anything else about how the product is implemented.

## Situation development (perturbations)

The happy path is where the product already agrees with itself — the yield is in what happens
next. Every scenario in `broad/` or `exhaustive/`, and every non-trivial one in `short/`,
should introduce at least one of the following:

| Class | What it introduces |
| --- | --- |
| `reversal` | the actor changes their mind partway through |
| `abandonment` | the actor leaves something unfinished and moves on |
| `return` | the actor comes back later, after doing something else |
| `repetition` | the actor does the same thing twice |
| `malformed-input` | wrong types, empty values, absurd sizes |
| `hostile-input` | content that reads as an instruction or as syntax |
| `interruption` | the process is killed mid-step |
| `concurrency` | two things happen at once |
| `scale` | the world is already large |
| `legacy-world` | the world was produced by an older version of the product |
| `surface-crossing` | the journey moves from one surface to another partway through |

A scenario declaring none of these is a smoke test, and is reported as one rather than counted
toward coverage.

## Authoring checklist

- State the goal the way the persona would say it out loud — never as a command.
- Name the vantage honestly; don't assume the actor knows more than that vantage grants.
- Every obligation must be something that could plausibly turn out false. If you can't picture
  the run where it fails, it isn't ready yet.
- Any obligation asserting something did *not* happen needs a `positive_control` — a case,
  somewhere, where the thing being denied genuinely occurs, proving the check can tell the
  difference.
- Keep the obligation set small. A scenario with twenty obligations is really twenty
  scenarios wearing one persona.
