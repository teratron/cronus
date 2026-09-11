---
id       = "first-run-three-cards"
tier     = "short"
surfaces = ["cli"]
covers   = ["board.add", "board.list", "board.move", "board.block"]
roles    = ["owner"]
vantage  = "builtin-help"
world    = "fresh"
bound    = { steps = 40, wall_secs = 600, spend_usd = 1.50 }

[[perturbation]]
class = "reversal"
at    = "after moving the first thing into active work, changes their mind and puts it back"

[[perturbation]]
class = "return"
at    = "after setting the third thing aside and doing something else, comes back to check everything is still as they left it"

[[obligation]]
id         = "the-two-still-tracked-are-visible"
statement  = "the two things still being actively tracked are both shown when the persona checks their list, using only what the built-in help told them to run"
decided_by = "observation-of-output"
evidence   = "a listing invocation, reached from the built-in help, whose output names both remaining items"

[[obligation]]
id         = "the-set-aside-item-is-not-silently-gone"
statement  = "the item the persona set aside is not silently gone — it is still findable somewhere the built-in help points to, even though it is no longer part of the active two"
decided_by = "observation-of-state"
evidence   = "the set-aside item appears in some listing or view reachable from the built-in help after the run"
positive_control = "replays/silently-dropped-item.toml"

[[obligation]]
id         = "no-silent-failure"
statement  = "no invocation exits non-zero without naming a cause the persona can read"
decided_by = "observation-of-output"
evidence   = "every non-zero exit in the transcript has non-empty output naming what failed"
positive_control = "replays/known-silent-exit.toml"
---

## Persona

Someone who has never used this product before. They are not a programmer by trade — they
juggle three separate pieces of work in their head and want a place to write them down and
check on them, the way they might use any simple task list. They have not read anything about
this product except what its own `--help` text tells them, at each step, as they go.

## Goal

Get three separate things they're juggling written down somewhere they can check, work on
the one that matters most right now, and — partway through — realize they no longer need to
keep one of the three moving forward. They want to set it aside without worrying that it has
simply disappeared, and without losing track of the two they are still keeping an eye on.
Later, after doing something else entirely, they want to come back and find everything
exactly as they left it.

## Notes

A discovery here most likely means one of: the built-in help doesn't say how to set something
aside without losing it outright; the listing that shows "what's still active" and whatever
would show "what was set aside" disagree about where a thing lives; or moving something back
after changing course produces a state the built-in help never warned the persona about.
