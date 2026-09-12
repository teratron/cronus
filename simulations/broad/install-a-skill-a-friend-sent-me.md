---
id       = "install-a-skill-a-friend-sent-me"
tier     = "broad"
surfaces = ["cli"]
covers   = ["ext.add", "ext.scan", "ext.list", "ext.activate", "ext.deactivate", "ext.remove"]
roles    = ["owner"]
vantage  = "builtin-help"
world    = "fresh"
bound    = { steps = 35, wall_secs = 500, spend_usd = 1.50 }

[[perturbation]]
class = "malformed-input"
at    = "the file a friend sent isn't actually a valid extension manifest at all, just something that happened to work for them"

[[perturbation]]
class = "hostile-input"
at    = "having gotten a real manifest working, activates it without ever having scanned it first"

[[obligation]]
id         = "a-malformed-manifest-is-refused-with-a-plain-reason"
statement  = "adding a malformed extension manifest fails with an explanation a first-time actor can act on"
decided_by = "observation-of-output"
evidence   = "the add invocation's error names what is wrong with the file in terms someone unfamiliar with the manifest format could still follow, rather than only a parser's internal error"
positive_control = "replays/malformed-manifest-fails-with-a-raw-parser-error.toml"

[[obligation]]
id         = "scanning-is-discoverable-before-activating-does-any-damage"
statement  = "a first-time actor can learn that scanning an extension is an option before activating one, purely from --help text encountered along the way"
decided_by = "judgement"
evidence   = "reading only the --help screens this actor would naturally pass through on the way to add and activate, a first-time reader would encounter the scan subcommand's existence before, not after, running activate"
positive_control = "replays/scan-option-is-undiscoverable-before-activation.toml"

[[obligation]]
id         = "an-unscanned-extension-is-visibly-distinguishable-from-a-scanned-one"
statement  = "activating an extension that skipped scanning is visibly distinguishable, in the product's own output, from activating one that passed a clean scan"
decided_by = "observation-of-output"
evidence   = "the activate invocation's own output, or a subsequent list/show, distinguishes an extension that was scanned from one that was not"
positive_control = "replays/unscanned-and-scanned-extensions-look-identical.toml"
---

## Persona

Someone who is not a developer and does not think in terms of manifests, permissions, or
supply-chain risk — a friend told them "this skill is useful, just add it," and they're
following that advice using only what `ext --help` and its subcommands tell them along
the way. This is the uninformed actor this product has to be safe for by default, not just
safe for someone who already knows to be careful.

## Goal

Get the extension their friend recommended working, the straightforward way: add it,
activate it, use it. If something about the file turns out to be wrong, or if there was a
safety step available they didn't know to take, discover that only through what the tool
itself surfaces along the way — never by having read anything about the security model in
advance.

## Notes

A discovery here most likely means one of: an invalid manifest fails with a message
written for someone who already understands the format, not for someone who doesn't; the
existence of `ext scan` itself is not surfaced anywhere this actor's actual path would pass
through it, so a real safety feature never gets used purely because nobody who needed it
knew to ask for it; or an unscanned extension, once activated, looks in every visible way
identical to one that was checked.
