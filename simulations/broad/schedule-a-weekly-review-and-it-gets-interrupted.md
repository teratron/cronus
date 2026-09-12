---
id       = "schedule-a-weekly-review-and-it-gets-interrupted"
tier     = "broad"
surfaces = ["cli"]
covers   = ["schedule.add", "schedule.list", "schedule.run", "schedule.delete"]
roles    = ["owner"]
vantage  = "builtin-help"
world    = "fresh"
bound    = { steps = 35, wall_secs = 500, spend_usd = 1.50 }

[[perturbation]]
class = "interruption"
at    = "the process running a fired schedule is killed partway through, before it reports back"

[[obligation]]
id         = "an-added-schedule-is-listed-with-its-real-recurrence"
statement  = "a schedule just added is described in the listing with the same recurrence it was given, rather than a default or placeholder value"
decided_by = "observation-of-output"
evidence   = "the listing entry for the newly added schedule names the same recurrence or preset that was passed to add"

[[obligation]]
id         = "a-schedule-killed-mid-fire-neither-vanishes-nor-double-counts"
statement  = "a schedule whose fired run was interrupted before completion is still present afterward, in a state that reflects the interruption rather than a clean success"
decided_by = "observation-of-state"
evidence   = "listing the schedule after the interrupted run shows it still exists and does not report the interrupted attempt as a normal completion"
positive_control = "replays/interrupted-schedule-silently-marked-complete.toml"

[[obligation]]
id         = "deleting-a-schedule-stops-it-from-firing-again"
statement  = "once a schedule has been deleted, firing it again by its old id is refused rather than silently running the deleted definition"
decided_by = "observation-of-output"
evidence   = "attempting schedule run against a deleted schedule's id fails, naming the schedule as no longer existing"
positive_control = "replays/deleted-schedule-still-fires.toml"
---

## Persona

Someone setting up a recurring habit they don't want to have to remember to trigger by
hand — a weekly review, in this case — the way anyone sets up a repeating reminder, working
only from `schedule --help`.

## Goal

Add a weekly recurring item, confirm it's registered the way they intended, fire it once by
hand to see what happens — and, since real processes get killed by real machines going to
sleep or crashing, have that one firing die partway through rather than finish cleanly.

## Notes

A discovery here most likely means the schedule's own bookkeeping has no notion of a run
that started but never reported back, so an interrupted attempt is either invisible (as if
it never happened, risking a genuine miss nobody notices) or wrongly counted as a success
(risking the opposite: a real problem hidden behind a green checkmark).
