# Derived Instructions

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Derived instructions is the rule that **the document telling an agent how to use a surface is generated from that surface, not written beside it**.

Every agent-facing capability ships with prose: a skill file, a tool description, a command reference, a "common errors" section. That prose is a *claim about the implementation* — these are the operations, these are their arguments, this is the message you will see when you get it wrong. Hand-authored, it is correct on the day it is written and decays from then on, silently, in the one direction that matters: the agent keeps following instructions for a surface that no longer exists, and the failure it produces looks like a confused model rather than a stale document.

The remedy is the one [l1-declarative-configuration.md](l1-declarative-configuration.md) already applies to configuration editors, moved one level up: the owner publishes a single machine-readable declaration, and the artifact humans and agents read is **rendered from it**. Here the artifact is the instruction, and the load-bearing extra is the message catalog — the exact strings the surface emits are the exact strings the instruction quotes, from one source, pinned by tests.

## Related Specifications

- [l1-declarative-configuration.md](l1-declarative-configuration.md) — the same shape one level down: DC-1's single declaration and DC-2's generated editing surfaces. This spec applies that discipline to instructions instead of editors, and DC-7's uniformity (built-in and external components use one mechanism) carries over unchanged.
- [l1-surface-parity.md](l1-surface-parity.md) — the behavioral sibling. Parity keeps surfaces from behaving differently; this keeps their descriptions from drifting from their behavior. SP-11's action catalog is the artifact both consume.
- [l1-agent-tool-ergonomics.md](l1-agent-tool-ergonomics.md) — ATE-2's success-shaped guidance and ATE-13's two rejection kinds define *what* a message must say; DIN-3/DIN-7 make the instruction quote exactly those messages rather than a paraphrase.
- [l1-extensions.md](l1-extensions.md) / [l2-skill-system.md](l2-skill-system.md) — skills are the largest population of generated instruction artifacts; the generation pipeline and the two-tier store are where DIN-1/DIN-2 land concretely.
- [l1-project-vocabulary.md](l1-project-vocabulary.md) — one canonical term per concept. A generated instruction inherits the vocabulary mechanically instead of re-inventing names per document.
- [l1-error-reporting.md](l1-error-reporting.md) / [l1-log-legibility.md](l1-log-legibility.md) — the message catalog serves the diagnostic surfaces too; one string, several audiences, is cheaper than several strings for one condition.
- [l1-progressive-disclosure.md](l1-progressive-disclosure.md) — instructions are disclosable artifacts; DIN-6's self-location is how an agent reaches the current one instead of a copy it was handed.
- [l1-invariant-tripwires.md](l1-invariant-tripwires.md) — DIN-4's contract tests and DIN-2's edit refusal are tripwires for rules a reviewer cannot reliably check by reading.
- [l1-staged-rollout.md](l1-staged-rollout.md) — DIN-5's capability probe is how an instruction covers an experimental surface without asserting it is present.

## 1. Motivation

**Instructions decay in exactly the direction that hurts.** A stale instruction rarely describes something that does not exist — it describes something that *changed*: an argument renamed, a default flipped, an operation split in two. The agent follows it, the call is refused, and the refusal reads as a model mistake. Nobody looks at the document, because the document is not in the failure path.

**The agent trusts the instruction more than the surface.** A human reading a manual that contradicts the program believes the program. An agent does the opposite: the instruction is in its context, authoritative and cheap, while the truth requires a call it has just been told not to make. A wrong instruction therefore does not merely fail to help — it actively steers away from the working path, and a small number of such failures teaches abandonment of the whole capability (ATE-2).

**Quoted failure messages are the fastest-rotting sentences in any document, and the highest-value.** A "common errors" section is what turns a dead end into one corrected call — but only while the strings match. The moment the emitted message is reworded, the instruction is quoting a message the agent will never see, and the correspondence that made the section useful is gone with no visible symptom.

**One condition, several audiences, several copies.** The same failure needs a sentence for a log, a sentence for a user-facing surface, and a sentence for an agent's instruction. Written independently, they diverge in *content*, not just in tone: three descriptions of one condition, each maintained by whoever last touched its surface, none of them wrong enough to notice.

**Conditional capabilities make documents lie confidently.** An instruction covering an experimental or optional capability must say *when* the capability is present. Hand-authored, it says "you can do X" and the agent tries X against a build that lacks it — and receives an error that describes a nonexistent problem, because the actual problem is that the document assumed a capability it never checked.

## 2. Constraints & Assumptions

- **Agent-facing instructions are a product surface.** They ship, they version, and they are read by something that acts on them; they are not developer documentation.
- **Generation covers description, not judgment.** What the operations *are* is derivable. When to use them, in what order, and what makes a good result is authored — and it lives in the generator's source, not in the generated file.
- **A build step exists.** The discipline needs a generation step in the pipeline; a project without one cannot hold this contract by convention.
- **Some instructions describe surfaces the project does not own.** For an external capability, the declaration may have to be captured rather than published; the contract then applies to the captured declaration.
- **Message catalogs are per-language-family, not per-surface.** Localization of a message is a translation of one catalog entry, never a second entry.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **DIN-1 (Instructions are generated from the surface's own declaration):** the agent-facing instruction for a capability is **rendered from the machine-readable declaration** of that capability — its operations, arguments, targets, constraints, and outcomes — and never maintained as an independent document that happens to describe it. The declaration is the source of truth; the instruction is an artifact.

- **DIN-2 (A generated artifact is not an editing surface):** a generated instruction **states its generator and its edit points**, and hand-editing it is a defect, not a shortcut. Every generated artifact carries the marker; the pipeline treats an edited artifact as a failure rather than regenerating over it silently. An artifact that can be edited *and* regenerated will be edited, and the edit will be lost at the least convenient moment.

- **DIN-3 (Every quoted message is single-sourced with its emitter):** the user- and agent-visible messages a surface emits live in **one catalog**, and every consumer — the emitter, each surface that displays it, and the instruction that quotes it — reads from that catalog. A message paraphrased into an instruction is a fork with no notification channel: the emitter's wording moves and the quote stays.

- **DIN-4 (The correspondence is pinned by contract tests):** an automated check asserts that the strings the instruction promises are the strings the surface actually emits, and that every operation the instruction describes exists with the arity and argument names given. The claim "the document matches the code" is exactly the kind of rule that is true at review time and false three commits later, so it carries a mechanical check rather than a convention.

- **DIN-5 (Conditional capability is gated on a runtime probe, never on the document's assumption):** where an instruction covers a capability that may be absent — experimental, optional, licensed, host-dependent — the surface **publishes its live capability set**, and the instruction directs the reader to **check it before using the capability**. A document that asserts an optional capability is present produces a failure describing a problem the agent does not have.

- **DIN-6 (The surface locates its own instructions):** a capability can be asked **where its current instructions are**, and that answer is the one an agent is directed to. Instructions travel as copies — pasted into configurations, bundled into other products, cached in contexts — and every copy is a version. Self-location gives one authoritative answer that cannot silently be a year old.

- **DIN-7 (Failures are catalog entries carrying their correction):** each failure an agent can provoke is a catalog entry with an identity, the emitted message, and the **corrective action**, and the instruction renders the set from that catalog. Both halves are load-bearing: without the identity the message cannot be matched across surfaces or localized, and without the correction the entry announces a dead end where it could have produced one corrected call (composing ATE-2/ATE-13).

- **DIN-8 (A description-invalidating change fails the build, not the agent):** changing a surface in a way that invalidates its instruction — removing an operation, renaming an argument, rewording a quoted message — **fails the pipeline**. The failure belongs at the change, where the information is; deferred, it is discovered by an agent in a session, reported as a model problem, and diagnosed by someone with none of the context the author had.

- **DIN-9 (Authored guidance lives in the generator, not in the artifact):** the judgment half — when to reach for this, the ordering, the economy rules, what a good result looks like — is genuine authorship and is **kept in the generator's source**, shipped through the same pipeline as the derived half. It is edited in one place, reviewed as source, and cannot be lost to a regeneration. An instruction is not a template with prose glued on afterwards.

- **DIN-10 (One description of an action, however many surfaces present it):** an action's identity, its human-readable name, and its current binding are read from the **shared action catalog** (SP-11) by every presenter — menus, help, palettes, reference docs, and agent instructions alike. A second description written for one surface is a second product vocabulary, and it drifts against the first from the day it is written.

## 4. Detailed Design

### 4.1 The two halves of an instruction

```
              ┌───────────────────────────────┐
declaration ─▶│  derived half                 │
(operations,  │  operations, arguments,       │
 arguments,   │  targets, constraints,        │──┐
 outcomes)    │  failures + corrections       │  │
              └───────────────────────────────┘  │      ┌──────────────────┐
                                                 ├─────▶│ generated        │
              ┌───────────────────────────────┐  │      │ instruction      │
generator ───▶│  authored half (DIN-9)        │──┘      │ artifact         │
  source      │  when to use it, ordering,    │         │ (never edited)   │
              │  economy, what good looks like│         └──────────────────┘
              └───────────────────────────────┘
                                                        ┌──────────────────┐
message ─────────────────────────────────────────┬─────▶│ emitter          │
catalog (DIN-3)                                  ├─────▶│ user surfaces    │
                                                 └─────▶│ the quotes above │
```

The catalog feeding *both* the emitter and the instruction is what makes DIN-4's contract test possible at all: with two sources there is nothing to compare, only two things to hope about.

### 4.2 What decays without each invariant

| Missing | Symptom | Where it is diagnosed |
| --- | --- | --- |
| DIN-1 | instruction describes last quarter's arguments | in a session, as a model mistake |
| DIN-3 | "common errors" quotes messages nobody emits | never — the section just stops helping |
| DIN-4 | the drift exists between two green test suites | in production |
| DIN-5 | agent uses an absent capability confidently | as an unexplainable error |
| DIN-6 | three copies of an instruction, one current | when two agents behave differently |
| DIN-8 | the breaking change ships, the doc follows later | by whoever is on call, without context |

Every row shares one property: the diagnosis happens far from the change, at a moment when the person diagnosing has the least information. That distance is the cost this spec removes.

### 4.3 The message catalog

An entry is small and has exactly the fields the three consumers need:

| Field | Consumed by |
| --- | --- |
| **identity** | matching across surfaces, localization, telemetry aggregation |
| **message** | the emitter, and the instruction's quote of it |
| **correction** | the instruction, and any surface offering a next step |
| **audience shaping** | optional per-surface phrasing of the *same* entry — never a second entry |

The last row is the one that gets violated first. A log line, a user-facing dialog, and an agent instruction legitimately word a condition differently; they must remain one entry with shaped renderings, or the condition acquires three independent descriptions and the identity that tied them together is gone.

### 4.4 Capability probes over assertions (DIN-5)

The generated instruction for an optional capability says, in effect: *this capability exists in some builds; ask the surface whether it exists in yours; here is how to use it if it does.* That is three sentences instead of two, and it converts a class of confident failures into a check.

It also makes staged rollout cheap. A capability behind a flag needs no separate instruction and no instruction edit when it graduates: the probe answers differently, and the same document is correct in both states.

### 4.5 Nodus relevance

Nodus is unusually well-placed here, and one gap is worth naming.

- **NL-1 (schema-first)** and **NL-16 (selective vocabulary disclosure)** already give the language a machine-readable declaration of its own command vocabulary — the exact input DIN-1 requires. A workflow-authoring instruction for an agent is therefore a *derivation of the schema*, not a hand-maintained parallel document, and DIN-8's build-time failure follows naturally from NL-1's validation being schema-driven.
- **NL-20 (`§config` as a validated declarative configuration surface)** is the DC-1 analogue inside the language; its declared fields are the same source a generated instruction would render.
- **The gap**: nodus's validation and runtime failures are the agent's primary feedback channel, and DIN-3/DIN-7 apply to them directly — a single catalog of diagnostic identities, messages, and corrections, consumed by the validator, the runtime, the host's surfaces, and any generated authoring instruction. This is an implementation obligation of the existing error contracts rather than a new language invariant, and it is recorded here so the catalog is built once rather than per-surface.

## 5. Implementation Notes

- Build the message catalog **before** the generator. A generator over hard-coded strings produces an artifact that looks right and satisfies nothing — the contract test has no second source to compare against.
- Make the regeneration check part of the ordinary gate, and make its failure message say *which declaration changed and what to run*. A pipeline failure that only says "generated file is stale" trains people to run the command without reading the diff, which is how a wrong change gets blessed.
- Keep the authored half (DIN-9) in a form that reviews as prose. If it becomes template fragments interleaved with logic, it stops being edited and the instruction quietly degrades to a reference sheet.
- Resist per-surface message overrides that duplicate an entry instead of shaping it. The pressure is real and constant, and the duplicate is invisible until a condition is renamed in one place.
- Where an instruction covers an external capability the project does not own, capture the declaration into the pipeline and check the capture; the contract then holds against the capture, and drift in the external surface becomes a visible capture failure rather than an invisible instruction lie.

## 6. Drawbacks & Alternatives

- **A generation step is real infrastructure.** It must run in the gate, in the release, and locally; when it is broken, instruction work stops entirely. That is the trade for the failure landing at the change rather than in a session.
- **Generated prose is duller.** A hand-written instruction can be sharper for its one moment of correctness. DIN-9 keeps the sharp half authored; the derived half is deliberately mechanical, because that is the half that has to stay true.
- **The contract test can only pin what it can compare.** It catches renamed operations and reworded messages; it cannot catch guidance that has become bad advice while remaining accurate. That failure remains a review problem.
- **Alternative — review discipline ("update the docs with the code"):** rejected. It is the rule this spec exists because everyone already has and nobody keeps; the drift has no symptom at the moment of the change.
- **Alternative — have the agent read the implementation instead of an instruction:** rejected. It spends a large context budget re-deriving what a declaration already states, and it produces a different derivation each time — the opposite of a stable surface contract.
- **Alternative — generate the instruction on demand from the live surface:** attractive and partially adopted through DIN-5/DIN-6. Rejected as the sole mechanism: it makes the instruction unreviewable before release, and the authored half (DIN-9) has to come from somewhere anyway.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[DECLCONFIG]` | `.design/main/specifications/l1-declarative-configuration.md` | DC-1/DC-2 — the same discipline one level down |
| `[PARITY]` | `.design/main/specifications/l1-surface-parity.md` | SP-11 action catalog, consumed by DIN-10 |
| `[ERGONOMICS]` | `.design/main/specifications/l1-agent-tool-ergonomics.md` | ATE-2/ATE-13 — what a catalog entry's correction must say |
| `[SKILLS]` | `.design/main/specifications/l2-skill-system.md` | The largest population of generated instruction artifacts |
| `[VOCAB]` | `.design/main/specifications/l1-project-vocabulary.md` | The naming a generated instruction inherits mechanically |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-13 | Core Team | Initial concept: **the document telling an agent how to use a surface is generated from that surface**, because a hand-authored instruction decays in the one direction that hurts and an agent trusts it over the surface itself. Instructions rendered from the capability's own declaration (DIN-1); a generated artifact **is not an editing surface** — it states its generator and its edit points, and an edited artifact fails rather than being silently regenerated over (DIN-2); **every quoted message single-sourced with its emitter** through one catalog, since a paraphrase is a fork with no notification channel (DIN-3); the correspondence **pinned by contract tests**, because "the doc matches the code" is true at review time and false three commits later (DIN-4); conditional capability **gated on a runtime probe**, never on the document's assumption, so an absent capability produces a check instead of a confident failure (DIN-5); the surface **locates its own instructions**, since every copy is a version (DIN-6); failures are **catalog entries carrying their correction**, identity and correction both load-bearing (DIN-7); a description-invalidating change **fails the build, not the agent**, keeping diagnosis at the change where the information is (DIN-8); **authored guidance lives in the generator**, shipped through the same pipeline, so judgment is neither lost to regeneration nor glued onto a template (DIN-9); **one description of an action** across menus, help, palettes, docs, and instructions, from the shared catalog (DIN-10). §4.2 tabulates what decays without each invariant and where it is diagnosed; §4.3 the catalog entry shape; §4.5 records the nodus disposition — NL-1/NL-16/NL-20 already supply the declaration, and the open obligation is one diagnostic catalog shared by validator, runtime, host surfaces, and generated authoring instructions. |
