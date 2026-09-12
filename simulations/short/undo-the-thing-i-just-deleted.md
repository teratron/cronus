---
id       = "undo-the-thing-i-just-deleted"
tier     = "short"
surfaces = ["cli"]
covers   = ["memory.store", "memory.forget", "memory.search", "board.add", "board.archive"]
roles    = ["owner"]
vantage  = "builtin-help"
world    = "fresh"
bound    = { steps = 30, wall_secs = 300, spend_usd = 1.00 }

[[perturbation]]
class = "reversal"
at    = "deletes a memory entry and immediately realizes they meant to delete a different one, or meant to keep it"

[[obligation]]
id         = "the-delete-command-warns-before-the-point-of-no-return"
statement  = "removing a memory entry either asks for confirmation first or names, in its own output, that the action cannot be undone"
decided_by = "observation-of-output"
evidence   = "the forget invocation's own output, at the moment it runs, tells the actor this is irreversible before or as it happens — not only discoverable by trying to undo it afterward and failing"

[[obligation]]
id         = "a-mistaken-forget-cannot-be-walked-back"
statement  = "once a memory entry is forgotten, no command surfaced by --help brings back its original content"
decided_by = "observation-of-state"
evidence   = "searching for the forgotten entry's own former title or content afterward returns nothing, and no other subcommand's help text describes a way to recover it"
positive_control = "replays/forget-is-silently-recoverable.toml"

[[obligation]]
id         = "archiving-a-card-is-not-the-same-as-losing-it"
statement  = "a card moved to the archived state through board archive is still findable through some listing --help points to, unlike a forgotten memory entry"
decided_by = "observation-of-state"
evidence   = "an archived card still appears through some board subcommand's output after archiving, in contrast to memory forget's outcome above"
---

## Persona

An ordinary user who deletes things the way most people do: without reading a manual first,
assuming that if the tool doesn't stop them, it's probably fine — and finding out immediately
afterward whether that assumption was right.

## Goal

Clear out a couple of memory entries and a couple of board cards they no longer need, the
way anyone tidies up — and, in the middle of it, delete one memory entry they realize a
half-second too late they actually wanted to keep.

## Notes

A discovery here most likely means one of: `memory forget` gives no warning at all and
genuinely, permanently destroys the entry with no recovery path advertised anywhere in
`--help` — a real trap for anyone clearing things out casually; or, in the opposite and
also interesting direction, `board archive` turns out to behave more like `memory forget`
than its name suggests, quietly making a card just as unfindable as a deleted one.
