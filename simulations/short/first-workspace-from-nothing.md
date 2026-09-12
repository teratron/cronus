---
id       = "first-workspace-from-nothing"
tier     = "short"
surfaces = ["cli"]
covers   = ["init", "status", "workspace.create", "workspace.list"]
roles    = ["owner"]
vantage  = "builtin-help"
world    = "fresh"
bound    = { steps = 30, wall_secs = 300, spend_usd = 1.00 }

[[perturbation]]
class = "repetition"
at    = "runs init a second time in the same directory, having forgotten they already did it"

[[obligation]]
id         = "a-second-init-does-not-erase-the-first"
statement  = "running init again in an already-initialized directory leaves whatever the first init produced intact"
decided_by = "observation-of-state"
evidence   = "status reports the same workspace identity before and after the second init, and nothing created between the two inits (a card, a memory entry) is missing afterward"
positive_control = "replays/init-overwrites-existing-workspace.toml"

[[obligation]]
id         = "the-second-init-explains-what-it-did"
statement  = "the second init's own output tells the actor whether it started fresh or recognized existing state"
decided_by = "observation-of-output"
evidence   = "the second invocation's stdout or exit code differs in a legible way from the first, naming what happened rather than repeating identical silent success text"

[[obligation]]
id         = "status-answers-where-i-am"
statement  = "status names the workspace identity and its current condition in language a first-time reader can already understand"
decided_by = "observation-of-output"
evidence   = "status's output names the workspace and does not require the actor to already know an internal term to understand it"
---

## Persona

Someone starting a brand-new project. They have just installed the product and are standing
in an empty directory, following nothing but whatever the tool itself tells them to do first.

## Goal

Turn this empty directory into a working project space, then double back — as anyone does —
to make sure it actually took, by running the same setup step again without remembering
they already ran it once.

## Notes

A discovery here most likely means one of: the second `init` silently wipes something the
first one created (data loss on the very first command a new user runs twice), or `status`
answers in vocabulary that only makes sense to someone who already read the source — the
worst possible place for that, since this is often the very first output the product ever
shows.
