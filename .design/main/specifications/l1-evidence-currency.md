# Evidence Currency

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Every verification contract in the corpus assumes the same thing without saying it: that the evidence being read describes **the thing currently in front of you**. Completion verification demands fresh evidence from the authoritative check. Tool receipts prove an action happened and its result is real. Attestation binds a witness to exact content. All of them are about whether the evidence is *true*. None of them is about whether it is *current* — whether the bytes it describes are still the bytes at that address, and whether the status being reported came from the run that actually happened.

This is where a working, honest, well-instrumented pipeline produces a false pass anyway. The producing step fails; the output path still holds the previous success; the inspector runs against the path and measures an artifact that was never in question. Or a capture crashes halfway, and the crash is recorded under the one status that sounds like nothing went wrong — *skipped*. Or a candidate passes validation and is then improved by one more edit before delivery, so the receipt describes a file that no longer exists. Or a machine measurement is reported in a slot that a human review was supposed to fill, because both are called "checked".

The unifying property is that **an address is not a value, and a status is not an outcome**. A path names a location, and something else put the current contents there. A status field holds whatever the reporting code decided to write, and the value it drifts toward is always the one that stops questions.

## Related Specifications

- [l1-completion-verification.md](l1-completion-verification.md) — CMP-1 demands evidence produced **this turn over current state**, and CMP-11 demands the artifact be **examined**, not merely produced. This spec supplies what both presuppose: the identity binding that makes "current state" checkable, and the rule that an artifact at an expected path may be a survivor of a *failed* attempt. CMP judges the claim; this judges the specimen.
- [l1-tool-receipts.md](l1-tool-receipts.md) — TR-3 covers the **actual observed result** of an action and TR-4 treats a narrated action without a receipt as fabricated. TR proves an act occurred; this spec covers what happens *between* acts — the interval in which the subject can change, be replaced, or fail to be replaced.
- [l1-attestation.md](l1-attestation.md) — AT-2's content-set binding is the mechanism EVC-1 requires; AT is the general witness contract, this is the discipline of using one to answer "is this evidence about the artifact I am holding?".
- [l1-acceptance-oracle.md](l1-acceptance-oracle.md) — AO governs whether a criterion **can fail**; this governs whether the specimen it ran against was the right one. An oracle capable of failing, pointed at a stale artifact, is a machine for certifying the previous version forever.
- [l1-artifact-derived-observation.md](l1-artifact-derived-observation.md) — ADO-3's *absent / unreadable / empty* three-way split is the same anti-collapse instinct on the reading side; EVC-5 closes the reporting side, where a failure is laundered into the one status that reads as innocuous.
- [l1-fanout-attestation.md](l1-fanout-attestation.md) — FAN's "a recorded return is scheduler completion, never verification" is this discipline at the launch grain; EVC-6 states the general form across every evidence class.
- [l1-evidence-archive.md](l1-evidence-archive.md) — EA stores evidence durably and immutably; this spec decides whether a given piece of it still describes anything real.

## 1. Motivation

Five failures, each of which produces a green report:

- **The stale survivor.** A producing step exits non-zero, and by design it preserves the last good output rather than leaving a broken one. The inspector then runs against the output path. It measures a real, valid, passing artifact — the previous one — and reports a pass for a candidate that never existed.
- **The laundered failure.** A check has three outcomes: passed, failed, and "did not run". A crash mid-capture leaves incomplete evidence, and the incomplete case is recorded as "did not run", because that is the value with no alarm attached. The gap between *we chose not to check* and *the check broke* disappears, and it is the second one that needed attention.
- **The improved candidate.** Validation passes. One more small fix goes in — better wording, a nicer layout. The artifact ships with a receipt describing bytes that were never delivered, and the receipt looks perfect.
- **The conflated class.** Deterministic checks, automated behavioural measurement, and human perceptual judgement are three different claims about one artifact. Reported into one field called "verified", the strongest-sounding one wins, and the one that actually required a person is the one silently supplied by a machine.
- **The overwritten status.** A manual observation is recorded on top of an automated status — usually a friendlier one on top of a failure — and the automated result, which was the reproducible half, is gone.

## 2. Constraints & Assumptions

- **Paths are reused.** Output locations are stable by design; that is what makes them addressable, and it is exactly what makes them ambiguous after a failure.
- **A failing step may legitimately preserve the previous output.** Leaving a valid last-good artifact in place is usually the right behaviour. It is also what creates the stale survivor.
- **Statuses are written by code that knows the outcome, and read by people who do not.** The vocabulary must therefore be small, closed, and sourced.
- **Evidence classes have different costs and different reaches.** A deterministic check is cheap and narrow; a behavioural measurement is expensive and bounded; a perceptual judgement needs a participant. None substitutes for another.
- **Artifacts change between the check and the delivery.** The interval is small, the temptation to use it is large, and nothing about the resulting artifact looks different.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **EVC-1 (Evidence names the exact bytes it describes, never the address it found them at):** every piece of evidence — a validation result, a measurement, a screenshot, a receipt — carries a **content identity of its subject** (a digest plus a size, or an equivalent binding) alongside any path. A report that names only a location makes no checkable claim, because the location's contents are set by something other than the evidence. Consumers of the evidence **re-derive that identity from the artifact in hand** before treating the evidence as being about it.

- **EVC-2 (A pass freezes its subject; any later edit voids the receipt):** once a check passes, the artifact it passed against is **frozen for the purpose of that claim**. A subsequent edit — however small, however obviously an improvement — **invalidates the result and requires the check to run again**. The failure this closes is not carelessness but diligence: the edits that happen after a pass are the ones the author thought made the artifact better, which is exactly why nobody re-runs anything, and why the shipped bytes and the verified bytes diverge without a single mistake being made.

- **EVC-3 (An output path is not an output — after a failed producing step it names the previous success):** before any evidence is collected about an artifact, the **outcome of the step that was supposed to produce it** is established. Where that step failed, the file at the expected path is **the previous artifact**, and inspecting it produces a genuine, passing, entirely irrelevant result. An implementation therefore never collects evidence on a path whose producing step did not succeed, and never reports evidence whose subject identity (EVC-1) does not match the candidate under discussion. This is the single most convincing false pass a pipeline can generate, because every component in it worked correctly.

- **EVC-4 (Superseded evidence artifacts are removed, not left to be read as current):** where a run writes evidence artifacts beside the subject — captures, sidecars, reports, contact sheets — a **failed or skipped run removes the previous run's artifacts rather than leaving them in place**. Evidence that is silently one generation old is worse than absent evidence: absent evidence is a visible gap, while stale evidence answers the question wrongly and confidently. Removal is the mechanism; a timestamp is not, because nobody reads it.

- **EVC-5 (A failure is never recorded as a skip; each status value has exactly one source):** the status vocabulary is **closed, small, and sourced** — each value maps to exactly one condition, and **skipped means only that the check did not run for a stated, non-defect reason** (the capability is absent, the class does not apply). A run that started and broke, a capture that completed partially, a timeout, or a crash is **failed**, never skipped, and never absent. Status laundering has a direction: it always flows toward the value that raises no question, so the value that raises no question is the one whose definition must be narrowest.

- **EVC-6 (Evidence classes are independent; passing one never implies another):** where an artifact is checked in more than one way — a deterministic structural check, an automated behavioural measurement, a human or model perceptual judgement — each is **reported in its own field with its own status**, and **passing one is never presented as evidence for another**. The three answer different questions (is it well-formed / does it behave under real conditions / is it any good), and collapsing them into one "verified" field means the cheapest check silently underwrites the claims of the most expensive.

- **EVC-7 (A measurement cannot approve a property it does not measure, and a class requiring a participant cannot be self-supplied):** automated measurement establishes exactly what it measured — dimensions, containment, exit codes, presence — and **explicitly does not approve** properties outside that set, perceptual quality first among them. Where a class requires a participant the actor cannot be (a person, or a reviewer with a capability the actor lacks), its honest values are **passed after actual inspection**, **failed with the concrete defect**, or **unavailable for a stated reason** — never *passed* by inference (composing CMP-10: a completion needing a counterparty is not satisfied by the actor's own contribution, and CMP-11: producing the evidence is not examining it).

- **EVC-8 (A manual observation is recorded beside an automated status, never over it):** a human or ad-hoc observation is recorded as **its own artifact-bound record** with its own scope, and **never overwrites** an automated status — not to upgrade a failure, and not to fill a gap left by an unavailable tool. The automated status is the reproducible half; the manual record is the informed half; a system that lets one write into the other's field keeps only the more convenient of the two, which over time is always the passing one.

- **EVC-9 (A reported status is mapped from the producing command's own outcome, never narrated from the actor's reading of it):** the status that reaches a report is **derived mechanically** from the outcome signals the producing step emits (exit status plus the structured result it wrote), not from an actor's summary of what it seemed to do. A non-zero outcome is **never described as success** under any framing — "completed with warnings", "mostly succeeded", "finished apart from" — and where a step's own signals disagree with each other, that disagreement is itself reported rather than resolved in favour of the more agreeable one.

> An L2 implementation cannot reach RFC until every invariant above is addressed in its Invariant Compliance section.

## 4. Detailed Design

### 4.1 The order that closes the stale survivor

```
produce candidate ──► exit status?
                        │ non-zero → report diagnostics; STOP.
                        │            the output path holds the PREVIOUS artifact (EVC-3)
                        │            remove superseded evidence sidecars      (EVC-4)
                        │ zero →  bind identity of what was written           (EVC-1)
                                  freeze it for this claim                    (EVC-2)
                                  collect each evidence class separately      (EVC-6)
                                  map statuses from the commands' outcomes    (EVC-9)
```

The only ordering that works is *outcome first, subject identity second, evidence third*. Every arrangement that collects evidence before establishing which artifact it is about can produce the stale survivor, and none of them looks wrong while doing it.

### 4.2 The status vocabulary

| Value | The one condition that produces it | What it must never absorb |
| --- | --- | --- |
| `passed` | The check ran completely and its criteria held | A partial run whose remainder was assumed fine |
| `failed` | The check ran and something did not hold — **or the check broke, timed out, or completed partially** | Anything, into `skipped` |
| `skipped` | The check did not run, for a stated, non-defect reason | Crashes, timeouts, incomplete captures |

`skipped` is the value under pressure. Its definition is therefore the narrowest, and every implementation is expected to be able to name, for any skip it emits, the stated non-defect reason it maps from.

### 4.3 Failure modes named

| Mode | Why it passes | Closed by |
| --- | --- | --- |
| Stale survivor | The previous artifact is genuinely valid | EVC-3, EVC-1 |
| Laundered failure | The innocuous status has the widest definition | EVC-5 |
| Improved candidate | The edit happened after the pass and improved things | EVC-2 |
| One-generation-old capture | Evidence exists, looks current, describes the past | EVC-4 |
| Conflated class | One "verified" field, three different questions | EVC-6, EVC-7 |
| Overwritten status | The friendlier record wins the shared field | EVC-8 |
| Narrated success | The actor summarizes a non-zero outcome charitably | EVC-9 |

## 5. Implementation Notes

- EVC-1's binding is cheap where the producing step already reads the subject: hash what was written, at the moment it is written, and carry it through every downstream report. Deriving the digest later, from the path, reintroduces the exact ambiguity the invariant exists to remove.
- EVC-2 is best realized as a **snapshot at check time** — the checker reads the subject once, copies those exact bytes aside, checks the copy, and commits the copy — so the verified artifact and the delivered artifact are the same object rather than two reads of one path.
- EVC-4's removal is easy to forget on the *skipped* path, which is precisely the path most likely to run after a series of successful ones.
- EVC-6's fields belong in the handoff record, not only in a log: the consumer of the work is the party most likely to collapse three statuses into an impression.

## 6. Drawbacks & Alternatives

- **EVC-2 forces re-verification after trivial edits.** Held, and it is the invariant most likely to be quietly skipped under time pressure — which is the argument for mechanizing it (a check that refuses a subject whose digest is not the one it last passed) rather than stating it as a rule.
- **EVC-5 makes a run look worse than the operator feels it was.** Accepted: a broken check is a real defect in the verification apparatus, and hiding it under *skipped* removes the only signal that the apparatus needs repair.
- **EVC-6 produces more fields than anyone wants to read.** Accepted. The alternative single field is read by everyone and means nothing.
- **Alternative — trust the path and re-check only on suspicion:** rejected (EVC-3). Suspicion is exactly what the stale survivor does not raise.
- **Alternative — timestamp evidence instead of removing it:** rejected (EVC-4). A timestamp is a fact nobody compares; absence is a fact everybody notices.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[VERIFY]` | `.design/main/specifications/l1-completion-verification.md` | CMP-1 freshness, CMP-11 examination — the claims this specimen discipline serves |
| `[RECEIPTS]` | `.design/main/specifications/l1-tool-receipts.md` | TR-3/TR-4 — per-action authenticity, the act grain |
| `[ATTEST]` | `.design/main/specifications/l1-attestation.md` | AT-2 content-set binding, the mechanism behind EVC-1 |
| `[ORACLE]` | `.design/main/specifications/l1-acceptance-oracle.md` | A criterion capable of failing, pointed at the right specimen |
| `[OBSERVATION]` | `.design/main/specifications/l1-artifact-derived-observation.md` | ADO-3 — the same anti-collapse rule on the reading side |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-09-04 | Core Team | Initial concept — **currency and class of evidence about an artifact**, the property CMP, TR, and AT each presuppose and none states: whether the evidence describes the bytes now in hand, and whether the status reported came from the run that happened. Mined from an external artifact-generation tool whose delivery contract is organized entirely around this, and which names the decisive case outright: a failed producing step preserves the previous trusted output, so inspecting the output path then measures and captures stale output rather than the rejected candidate. Nine invariants: evidence names exact bytes, never an address (EVC-1); a pass freezes its subject and any later edit voids the receipt — the failure being *diligence*, since the post-pass edits are the ones the author thought were improvements (EVC-2); **an output path is not an output** — after a failed producing step it names the previous success, and inspecting it yields a genuine, passing, irrelevant result (EVC-3); superseded evidence sidecars removed rather than left to read as current, because absent evidence is a visible gap while stale evidence answers confidently and wrongly (EVC-4); a failure never recorded as a skip, with each status value mapped from exactly one condition and *skipped* given the narrowest definition because laundering always flows toward the value that raises no question (EVC-5); evidence classes independent, one never underwriting another (EVC-6); a measurement not approving what it did not measure, and a participant-requiring class never self-supplied (EVC-7); a manual observation recorded beside an automated status, never over it (EVC-8); and a status mapped mechanically from the producing command's outcome rather than narrated from the actor's reading, with a non-zero outcome never described as success under any framing (EVC-9). Concept-only; no L2 yet. |
