---
id       = "first-workflow-file"
tier     = "short"
surfaces = ["cli"]
covers   = ["workflow.scaffold", "workflow.validate", "workflow.run"]
roles    = ["owner"]
vantage  = "builtin-help"
world    = "fresh"
bound    = { steps = 40, wall_secs = 600, spend_usd = 1.50 }

[[perturbation]]
class = "repetition"
at    = "re-runs validate after editing the file by hand, more than once, trying to make the error go away"

[[perturbation]]
class = "abandonment"
at    = "gives up on the name they first chose and starts over under a different one instead of continuing to debug it"

[[obligation]]
id         = "a-freshly-scaffolded-file-validates-cleanly"
statement  = "a workflow file produced by scaffold, unedited, passes its own validator with a clean status"
decided_by = "observation-of-output"
evidence   = "validate's reported status is ok immediately after scaffold, with zero error[...] lines"

[[obligation]]
id         = "the-obvious-hand-fix-actually-works"
statement  = "editing the header field the error message names, to the value the error message itself implies is expected, results in a file that then validates"
decided_by = "observation-of-output"
evidence   = "after changing the header line to match what the error text describes as correct, a re-run of validate reports ok"

[[obligation]]
id         = "the-diagnostic-names-what-is-actually-wrong"
statement  = "the validator's error message describes the real cause of the mismatch it reports, rather than only a symptom of it"
decided_by = "judgement"
evidence   = "a reader comparing the error text against the file's actual content can identify why the two disagree, not just that they disagree"
---

## Persona

A developer setting up automation for their project for the first time. They know nothing
about the workflow file format beyond what `workflow scaffold --help` and `workflow validate
--help` tell them — no prior exposure to the language, no documentation beyond those two
screens.

## Goal

Generate a first workflow file under a name that actually describes what it does — the way
anyone naming a new file in a real project would, with a few descriptive words joined
together — get it to validate cleanly, and run it once to see it work.

## Notes

A discovery here most likely means one of: the scaffolded skeleton does not survive its own
validator untouched, which means every first-time author's very first command sequence ends
in an unexplained error before they have written a single line themselves; or the fix the
error message points toward does not actually fix anything, which is worse than no diagnostic
at all — it spends the author's trust and their time on a lead that goes nowhere, with only
`--help` text and the error itself to fall back on afterward.
