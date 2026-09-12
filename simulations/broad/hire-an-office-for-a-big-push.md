---
id       = "hire-an-office-for-a-big-push"
tier     = "broad"
surfaces = ["cli"]
covers   = ["role.list", "role.hire", "role.show", "role.fire", "board.add", "board.move"]
roles    = ["owner"]
vantage  = "builtin-help"
world    = "fresh"
bound    = { steps = 50, wall_secs = 700, spend_usd = 2.00 }

[[perturbation]]
class = "abandonment"
at    = "fires a role that still has a card assigned to it partway through, instead of finishing or reassigning that piece of work first"

[[perturbation]]
class = "scale"
at    = "hires several roles at once for a bigger push rather than one at a time"

[[obligation]]
id         = "the-preset-catalog-is-browsable-before-committing"
statement  = "the actor can see the full list of available role presets and what each one is for before hiring any of them"
decided_by = "observation-of-output"
evidence   = "a listing invocation, reachable from --help, names every preset with enough description to choose among them without hiring one first to find out"

[[obligation]]
id         = "hiring-several-roles-keeps-each-one-separately-addressable"
statement  = "hiring multiple role instances in the same session results in each one being separately addressable by its own name"
decided_by = "observation-of-state"
evidence   = "a listing after hiring several roles shows one distinct entry per hire, each independently addressable by the name it was given or assigned"
positive_control = "replays/second-hire-overwrites-the-first.toml"

[[obligation]]
id         = "firing-a-role-with-assigned-work-says-what-happens-to-that-work"
statement  = "firing a role instance that still has a card or task pointed at it either refuses until that link is cleared, or reports plainly what became of the link"
decided_by = "observation-of-output"
evidence   = "the fire invocation's own output addresses the still-assigned work rather than silently succeeding with no mention of it"
---

## Persona

Someone running a real piece of work who wants extra hands rather than doing everything
themselves — hiring a small office of role instances the way a lead might staff up a team
for a deadline, using nothing but `role --help` and its subcommands to figure out who's
available and how to bring them on.

## Goal

Look over who's available to hire, bring on several at once for a coordinated push, give
one of them a piece of tracked work, and then — mid-push, because priorities shifted —
let that role go before its assigned work is actually finished.

## Notes

A discovery here most likely means one of: the preset catalog is only discoverable by
already knowing a preset id to hire (a chicken-and-egg problem for a first-time office
builder), hiring several instances in one sitting produces ambiguous or colliding
identities, or firing someone mid-task leaves a card silently pointing at a role instance
that no longer exists, with nothing surfacing that fact until much later.
