---
id       = "remember-it-differently-than-i-stored-it"
tier     = "short"
surfaces = ["cli"]
covers   = ["memory.store", "memory.search"]
roles    = ["owner"]
vantage  = "builtin-help"
world    = "fresh"
bound    = { steps = 25, wall_secs = 300, spend_usd = 1.00 }

[[perturbation]]
class = "return"
at    = "tries to recall the stored fact weeks later, having naturally forgotten the exact words used to save it"

[[obligation]]
id         = "a-close-synonym-still-finds-the-entry"
statement  = "searching with a word that means the same thing as the one used at storage time, but is not that exact word, still surfaces the entry"
decided_by = "observation-of-output"
evidence   = "a search query built from a genuine synonym or closely related term for the stored title returns the entry rather than an empty result"
positive_control = "replays/synonym-search-returns-nothing.toml"

[[obligation]]
id         = "an-empty-search-result-suggests-a-next-step"
statement  = "a search that finds nothing tells the actor what to try differently, rather than only reporting absence"
decided_by = "observation-of-output"
evidence   = "the no-results output offers some guidance (the exact stored title, a hint to try the literal word, a suggestion to browse) beyond a bare negative"
---

## Persona

Someone using this as a place to jot down small facts they don't want to have to remember
themselves — a wifi code, an account number, a confirmation reference — the way anyone
uses a personal notes app, with no expectation of learning a query syntax.

## Goal

Save one such fact under whatever word felt natural at the time, then — weeks later, having
naturally not remembered the exact phrase they used — find it again using a different but
clearly related word for the same thing.

## Notes

A discovery here most likely means the search is a literal keyword match rather than
anything that understands meaning, so the exact scenario this tool exists for — writing
something down once and finding it later without perfect recall — fails on the very
first realistic instance of "later" a real user would hit. If that's what's found, the
second obligation becomes the load-bearing one: a search that comes back empty is only as
good as what it tells the person to try next.
