---
id       = "morning-triage-of-the-days-chores"
tier     = "short"
surfaces = ["cli"]
covers   = ["board.add", "board.list", "board.done", "board.move"]
roles    = ["owner"]
vantage  = "builtin-help"
world    = "fresh"
bound    = { steps = 35, wall_secs = 400, spend_usd = 1.00 }

[[perturbation]]
class = "return"
at    = "closes the terminal after the first morning and comes back the next morning to check the list again"

[[perturbation]]
class = "repetition"
at    = "checks the list a third time later the same day, after finishing one of the items"

[[obligation]]
id         = "an-empty-morning-reads-as-intentionally-empty"
statement  = "checking the list before anything has ever been added reads as an intentionally empty list rather than as an error or a hang"
decided_by = "observation-of-output"
evidence   = "the very first listing invocation, before any item exists, produces prompt, legible output rather than a bare error or silence"

[[obligation]]
id         = "yesterdays-unfinished-items-are-still-there-the-next-morning"
statement  = "an item added one day and left unfinished is still present and in the same state when the list is checked again the next day"
decided_by = "observation-of-state"
evidence   = "the second morning's listing names every item the first morning's listing named, with no state change neither morning's actions caused"

[[obligation]]
id         = "finishing-one-item-does-not-touch-the-others"
statement  = "marking one item done changes only that item's own status, leaving every other item's status exactly as it was"
decided_by = "observation-of-state"
evidence   = "the listing taken right after marking one item done differs from the listing taken right before it in exactly one item's state field"
---

## Persona

Someone using this tool the way they'd use any everyday task list — not for a software
project at all, just three or four ordinary things they need to not forget: reply to an
email, sort out a bill, pick up a package, call someone back. They read nothing beyond
`board --help` and its subcommands' own `--help` text.

## Goal

Write down today's handful of chores first thing in the morning, glance at the list to
decide what to do first, finish one of them, and — this being an ordinary daily habit, not
a one-time task — come back and do the exact same thing again the next day, expecting
whatever they didn't get to still be sitting there waiting.

## Notes

A discovery here most likely means one of: an empty list on the very first run reads
ambiguously (is nothing here because nothing was added, or because something failed?); a
day's carried-over item quietly loses information overnight (a reason, an assignee, its own
history) that the persona never asked to have touched; or marking one thing done has some
visible side effect on the others that a plain daily-chores mental model would never predict.
