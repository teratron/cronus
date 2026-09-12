# Usage Simulation

A practical guide to `cronus-sim`: the harness that drives the real `cronus` binary
toward a stated goal, the way an actual first-time user or a returning one would —
never by replaying a fixed script.

## Why this exists, in one paragraph

A scenario names *who wants what and what they're allowed to know* — never which
commands to type. Something (an agent today, a human later) drives the real product
toward that goal by whatever route it finds, because the routes nobody thought to
script are exactly the ones worth watching. Every judgement this harness produces —
pass, fail, or "still open" — is decided against a small, explicit set of
obligations the scenario declares up front, never against a transcript someone eyeballs
afterward.

Scenario files live under [`simulations/`](../simulations/); their format (frontmatter
fields, vantage levels, perturbation classes, the authoring checklist) is documented in
[`simulations/README.md`](../simulations/README.md). This guide covers the other half:
the `cronus-sim` command itself, and when to reach for it.

## Building it

```powershell
cargo build -p cronus-cli --bin cronus          # the product under test
cargo build -p cronus-simulation --bin cronus-sim
```

Both binaries land in `target/debug/` (or `target/release/` with `--release`).
`cronus-sim` finds `cronus` on its own by walking up from its own executable's
directory — no `--bin` flag or `PATH` setup needed when both were built the normal
way. Override it explicitly with `--bin <path>` (accepted by `world new` and
`coverage`), or by setting `CRONUS_SIM_BIN_CRONUS=<path>` in the environment.

## Core concepts

| Term | Meaning |
| --- | --- |
| **World** | A disposable, isolated working directory the product runs inside. Built fresh per scenario, torn down on `finish`. Never touches the real repository checkout — a run that somehow does is reported `contaminated`, not scored on its own terms. |
| **Bound** | Three independent ceilings the scenario declares: `steps` (how many `run` invocations), `wall_secs` (how much real time may elapse), `spend_usd` (how much the *driving agent itself* may spend exploring). Crossing any one of them refuses the next `run` — the mechanism that stops a stuck actor from spending without limit on a goal the product cannot satisfy. |
| **Obligation** | A statement about the world that must hold once the run ends — decided by a human or an agent watching the transcript, recorded with `verdict`, never inferred automatically. An obligation set that could never fail (empty, vague, an activity instead of an outcome) is refused when the scenario file is loaded. |
| **Note** | A free-text discovery, recorded with `note`. Never affects the computed outcome — it is information, not a check. |
| **Outcome** | What `finish` reports: `pass`, `fail`, `incomplete` (an obligation was never given a verdict), `contaminated` (the real repository was dirtied mid-run), `environment`/`void` (the harness itself failed to tear the world down cleanly — never blamed on the product). |
| **Replay case** | A transcript `pin` distills into a checked-in `.toml` file under `crates/simulation/tests/replays/`. Runs deterministically, with no agent and no judgement, in the ordinary `cargo test` gate — the cheap, permanent guard a one-time discovery buys once it has been made. |

## Command reference

| Command | What it does |
| --- | --- |
| `cronus-sim world new [--bin <path>] <scenario-file>` | Parses and validates the scenario, builds a fresh world, prints a world id. |
| `cronus-sim run <world-id> -- <argv...>` | Spawns `cronus <argv...>` inside the world, prints its stdout/stderr, and records the exit code. Refused once any bound is exhausted. |
| `cronus-sim note <world-id> <text...>` | Records a discovery. Always succeeds; never scored. |
| `cronus-sim spend <world-id> <usd>` | Reports the driving agent's own incremental cost since its last report (its own token spend, not anything the product can measure). Always succeeds, even past the bound — the refusal surfaces on the *next* `run`, the same way the wall-clock bound is discovered passively. |
| `cronus-sim verdict <world-id> <obligation-id> <pass\|fail> [--cite i,j,...]` | Records a verdict on one declared obligation, optionally citing transcript entry indices as evidence. |
| `cronus-sim finish <world-id>` | Computes the outcome, tears the world down, prints a report. Exit code encodes the outcome (see below). |
| `cronus-sim pin <world-id> <name>` | Distills every recorded `run` entry into `crates/simulation/tests/replays/<name>.toml`. **Must run before `finish`** — `finish` deletes the world, and `pin` needs it to still exist. |
| `cronus-sim coverage [--bin <path>]` | Prints the product's full action catalog, how much of it the scenario corpus's `covers` declarations reach, and — the actual headline — the list of what nothing covers yet. Never a percentage: a bare "60%" collapses "60% of six" and "60% of a hundred" into the same-looking number, which is not something anyone can act on. |

`finish`'s exit code: `0` pass, `1` fail, `2` incomplete, `3` contaminated, `4` void,
`5` a `cronus-sim` usage error, `6` a `run` refused by an exhausted bound.

## A full worked walkthrough

Driving the one scenario already in the corpus,
[`simulations/short/first-run-three-cards.md`](../simulations/short/first-run-three-cards.md):
a newcomer who has read nothing but `--help` juggles three cards, sets one aside without
losing it, and comes back later to find everything as they left it. What follows is a real
run, kept exactly as it happened — including the wrong turns, because the wrong turns are
the entire reason this harness exists rather than a fixed script.

```bash
WID=$(cronus-sim world new simulations/short/first-run-three-cards.md)

# Three things to track.
cronus-sim run "$WID" -- board add card-1 "write the proposal"
cronus-sim run "$WID" -- board add card-2 "review the budget"
cronus-sim run "$WID" -- board add card-3 "call the vendor"
cronus-sim run "$WID" -- board list        # all three start in `triage`
```

Work the most important one. `board move --help` lists all five states
(`triage|todo|ready|running|blocked|done`) as equally valid targets — nothing in it says
they form an ordered chain:

```bash
cronus-sim run "$WID" -- board move card-1 running
```

```text
error: invalid transition: Triage → Running (from Triage, only Todo is allowed)
```

A real discovery on the first real attempt — record it, then find the actual chain by
trying the next step `--help` implied was reachable:

```bash
cronus-sim note "$WID" "board move --help lists all five states as valid targets but never discloses the allowed order; triage->running is refused, discoverable only by trying"
cronus-sim run "$WID" -- board move card-1 todo      # ok
cronus-sim run "$WID" -- board move card-1 ready      # ok
cronus-sim run "$WID" -- board move card-1 running    # ok — the real chain is triage->todo->ready->running
```

Now the scenario's own "reversal": card-1 was just moved into active work, and the
persona changes their mind and puts it back. The forward chain suggests one step
back is `ready` — that turns out to be wrong too:

```bash
cronus-sim run "$WID" -- board move card-1 ready
```

```text
error: invalid transition: Running → Ready (from Running, only Todo, Blocked, Done are allowed)
```

```bash
cronus-sim note "$WID" "the chain is asymmetric: Running steps back to Todo (skipping Ready), not to Ready — the 'undo one step' intuition the forward chain suggests does not hold in reverse"
cronus-sim run "$WID" -- board move card-1 todo       # the real way back
```

Set the third card aside without losing it. `board block --help` says only "Block a card
with a reason" — nothing about which states it accepts:

```bash
cronus-sim run "$WID" -- board block card-3 "not needed right now"
```

```text
error: invalid transition: Triage → Blocked (from Triage, only Todo is allowed)
```

Following the same forward chain just learned — and finding it does not stop where a
reasonable guess would place it:

```bash
cronus-sim run "$WID" -- board move card-3 todo
cronus-sim run "$WID" -- board block card-3 "not needed right now"   # still refused: Todo allows only Triage, Ready
cronus-sim run "$WID" -- board move card-3 ready
cronus-sim run "$WID" -- board move card-3 running
cronus-sim run "$WID" -- board block card-3 "not needed right now"   # ok — Blocked is reachable only from Running
```

```bash
cronus-sim note "$WID" "MAJOR: Blocked is reachable only from Running — a card must be promoted through the whole todo->ready->running chain before it can be set aside at all; there is no way to block a card the persona has merely written down and never started"
```

Check what's left visible, do something else, then return later and confirm nothing
moved on its own:

```bash
cronus-sim run "$WID" -- board show card-2
cronus-sim run "$WID" -- board list
```

```text
id: card-1, state: todo
id: card-2, state: triage
id: card-3, state: blocked
```

Both still-active cards are visible, and the set-aside one is still listed with its
real state rather than gone — judge each declared obligation against exactly that
output, report the agent's own cost for the exploration, **pin the route before
finishing** (`finish` deletes the world, so a route can only be frozen while it still
exists), then close the world out:

```bash
cronus-sim spend "$WID" 0.06

cronus-sim verdict "$WID" the-two-still-tracked-are-visible pass --cite 19
cronus-sim verdict "$WID" the-set-aside-item-is-not-silently-gone pass --cite 19
cronus-sim verdict "$WID" no-silent-failure pass --cite 4,6,9,13

cronus-sim pin "$WID" three-cards-block-only-from-running

cronus-sim finish "$WID"
```

The obligations pass — nothing here disappeared and every refusal named its cause — but
the run still surfaced a real product behavior worth someone's attention: a not-yet-started
card cannot be set aside directly, only after being pushed all the way to `running`. That
is exactly the kind of finding a fixed test script, which only ever executes the route its
author already had in mind, cannot produce. The pinned file now runs, deterministically,
every time anyone runs `cargo test -p cronus-simulation` — the moment this behavior ever
changes, that test fails and says why.

## When to run what

Free-route exploration costs an agent's own time and money by design — that
unpredictability is the entire point, and it is exactly why it does not belong in an
automated gate. Only its frozen residue (a pinned replay) belongs there.

| Layer | Cost | Run it… | …where |
| --- | --- | --- | --- |
| Pinned replays (`crates/simulation/tests/replays/*.toml`) | deterministic, seconds | on every change | ordinary `cargo test`, already wired in |
| Scenario-file validation (`crates/simulation/tests/corpus.rs`) | deterministic, seconds | on every change | ordinary `cargo test`, already wired in |
| `cronus-sim coverage` | seconds | closing a phase of work, before cutting a release | run by hand, read as a report — never gated on |
| `simulations/short/` | an agent in the loop, minutes | right after a phase of work closes, before the next one is planned — the one window where a fresh discovery can still change what gets planned next | by hand |
| `simulations/broad/` | more turns, more cost | before a milestone or a release candidate | by hand |
| `simulations/exhaustive/` | the most expensive tier | on demand, ahead of a major release | by hand |

Never wire free-route exploration into CI. An agent driving toward a goal by whatever
route it finds is, by construction, not reproducible run to run and burns real tokens —
putting it in a pipeline would force it into a fixed script, which is the one thing a
scenario is defined not to be.

## Cleanup

A world's files live under the OS temp directory (`%TEMP%`/`$TMPDIR`), named
`cronus-sim-world-<id>` — never inside the repository checkout; the harness actively
refuses to build one there. `finish` deletes the directory as part of tearing the world
down. A world abandoned mid-run without ever calling `finish` (a crashed agent, a killed
process) simply leaves its directory behind — safe to delete by hand, and harmless to
leave, since nothing outside the temp directory ever depends on it.

## Known limits

- `spend_usd` is a budget on the *driving agent's own* cost, not on anything the
  product itself does — nothing here yet computes a dollar figure automatically. Until
  a scenario step involves an actual model call, `spend` has to be reported by hand (or
  by whatever is orchestrating the agent).
- Coverage is computed from the product's own shell-completion output, which only
  lists CLI actions today. A scenario exercising the terminal UI or the desktop shell
  still runs and is still judged; it just doesn't show up in `coverage`'s catalog count.
