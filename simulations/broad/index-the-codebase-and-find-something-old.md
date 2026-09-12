---
id       = "index-the-codebase-and-find-something-old"
tier     = "broad"
surfaces = ["cli"]
covers   = ["codegraph.index", "codegraph.search"]
roles    = ["owner"]
vantage  = "builtin-help"
world    = "seeded-small"
bound    = { steps = 30, wall_secs = 500, spend_usd = 1.50 }

[[perturbation]]
class = "legacy-world"
at    = "searches after the code has changed since the last index, without having re-indexed first"

[[obligation]]
id         = "a-fresh-index-finds-what-is-actually-there"
statement  = "searching right after indexing a real project surfaces symbols that genuinely exist in that project's own source"
decided_by = "observation-of-output"
evidence   = "a search for a name known to exist in the indexed tree returns a result naming that symbol, immediately after indexing"

[[obligation]]
id         = "a-stale-index-does-not-claim-currency-it-does-not-have"
statement  = "searching against an index that predates a real change to the underlying code either reflects the change or discloses that the index may be stale, rather than confidently reporting the old state as current"
decided_by = "judgement"
evidence   = "a reader comparing the search result against the code's real, current state can tell whether the tool is claiming freshness it doesn't have, versus honestly reporting what it indexed and when"
positive_control = "replays/stale-index-reported-as-current.toml"
---

## Persona

A developer joining an existing project rather than starting one — they want to understand
what's already there by asking the tool to find things, rather than reading every file by
hand, working only from `codegraph --help`.

## Goal

Index the real project once, confirm the index actually reflects what's in the code by
searching for something they know exists, then — realistically, since code keeps changing
after the first index — go looking for something again after the underlying files have
moved on, without having thought to re-index first.

## Notes

A discovery here most likely means the tool has no visible notion of its own index's age at
all, so a search against a codebase that has since changed reads exactly as confidently as
one taken the moment the index was built — the one distinction that actually matters to
someone trying to trust what it tells them.
