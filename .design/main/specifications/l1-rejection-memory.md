# Rejection Memory

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Work generates proposals continuously — a request, a suggested refactor, a feature idea, a candidate surfaced by a periodic survey. Most are accepted or reshaped. Some are **declined**, and the decision to decline is usually the most expensive one in the set: it took a real argument, it often took a person, and it is the one that leaves no artifact behind. The acceptance leaves code; the rejection leaves a closed item and a thread nobody rereads.

So the proposal comes back. A new session re-derives the same candidate from the same evidence. A different contributor asks for the same thing in different words. The survey that found it last month finds it again this month, because nothing it reads knows it was already settled. Each return costs the full argument again, and the argument gets weaker each time, because the reasons were sharpest when they were fresh.

This concept gives declined proposals a durable home: an entry **per concept**, carrying a reason written to be read cold, accumulating the requests that have arrived for it, consulted **before** a proposal is made rather than after one is filed. Two boundaries keep the store honest and are stated as invariants rather than left to good sense: a **deferral is not a rejection**, and a proposal declined because the thing already exists is not a rejection at all.

## Related Specifications

- [l1-negative-specification.md](l1-negative-specification.md) — the adjacent contract, and a deliberate boundary. NEG governs **exclusions that steer a generator**: what output must avoid. REJ records **decisions about scope**: which proposals were declined and why. An entry becomes an exclusion only by a second, deliberate act (REJ-12).
- [l1-pattern-codification.md](l1-pattern-codification.md) — codification promotes a repeated *practice* into a rule; this preserves a repeated *decision* against a proposal. Same instinct, opposite polarity.
- [l1-corpus-originality.md](l1-corpus-originality.md) — near-duplicate admission over a corpus of artifacts; REJ-4's concept matching is the same shape applied to proposals, and the two share the failure of matching on wording instead of meaning.
- [l2-trigger-triage.md](l2-trigger-triage.md) — the intake path a declined proposal exits through; the triage act is where REJ-3's already-satisfied split is drawn.
- [l1-review-checkpoint.md](l1-review-checkpoint.md) — RC-2's *reject* arm records a reason for one item; this is where that reason goes when it should outlive the item.
- [l1-operational-ledger.md](l1-operational-ledger.md) — the record plane an entry lives on; a rejection is ledger-grade fact, not conversational residue.
- [l1-improvement-loop.md](l1-improvement-loop.md) — the loop that re-derives candidates. Without REJ-7 it re-derives declined ones forever and reports them as findings.
- [l1-solution-frugality.md](l1-solution-frugality.md) — the disposition that declines work; REJ preserves the specific declines so frugality does not have to be re-argued from first principles each time.
- [l1-change-attribution.md](l1-change-attribution.md) — REJ-11's authority rule composes attribution: an entry names who decided, because an entry written by the party that would otherwise do the work is self-serving.

## 1. Motivation

Every failure here is a consequence of storing a decision in the act that closed it rather than in a place the next proposer reads:

- **The re-proposal loop.** A survey capability run on a cadence re-derives the same candidate every run, because the evidence that produced it has not changed. The reader learns to skim the survey, which destroys the value of the candidates that are new.
- **Decision amnesia.** The reason was stated once, in a thread. Six months later the thread is unreachable in practice, and the concept is re-argued by people who have no access to the argument that settled it.
- **Contamination by false rejection.** A request closed because the capability *already exists* gets filed alongside genuine rejections. Later matching then declines a proposal on the grounds that it was previously rejected, when in fact it was previously **delivered** — the worst possible failure of a store whose only job is to be trusted.
- **A deferral frozen into a rejection.** "Not now" and "no capacity this quarter" are circumstances, and they expire. Recorded as rejections, they outlive the circumstance and permanently close an idea nobody actually decided against.
- **Invisible demand.** Five independent requests for one concept, filed and closed separately, each look like a single voice. The one signal that most reliably indicates a rejection is due for reconsideration — that people keep arriving at it independently — is the signal the per-request record destroys.

## 2. Constraints & Assumptions

- **A rejection is a decision, not a fact.** It can be wrong, and it can stop being right. The store is therefore designed to be **retired from**, not only appended to.
- **Concepts are recognizable across wordings.** Matching a new proposal to a prior decision requires reading for meaning, not string overlap; where that judgement is unavailable the store degrades to a list a person reads, which is still better than nothing.
- **The set stays small.** A rejection store is valuable because it is short enough to consult. It records declined *concepts*, of which a project accumulates few, not declined *requests*, of which it accumulates many.
- **Entries are read by strangers.** The audience is a future proposer with none of the original context — human or otherwise — so an entry that only makes sense to the person who wrote it has failed.
- **The store is local and principal-owned**, consistent with the project's authority plane; nothing here creates an outward channel.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **REJ-1 (Keyed by concept, never by request):** the unit of the store is a **declined concept**, not a declined request. Repeat arrivals of the same concept attach to the existing entry rather than creating a second one. One-entry-per-request produces a store that grows with traffic, stops being consultable at exactly the volume where it would start to pay, and destroys REJ-6's demand signal by scattering it.

- **REJ-2 (A deferral is not a rejection):** an entry records a **durable** reason — one that holds independent of when it is read. "Not now", "no capacity", "not this cycle", "worth it later" are **deferrals**: statements about circumstance, which expire. Recording a deferral as a rejection permanently closes something nobody decided against, and does so invisibly, because the entry reads exactly like a real one. Deferrals belong in the planning plane, where they can come back; the store refuses them.

- **REJ-3 (Already-satisfied is not rejected):** a proposal declined because the capability **already exists** is closed by **pointing at where it lives**, and MUST NOT produce an entry. It is a delivered feature, not a declined one. An entry recorded here contaminates future matching with a false rejection, so a later request for something the project *has* is met with "we decided against this" — the store confidently misinforming a reader about the project's own contents.

- **REJ-4 (Matched by concept, not by wording):** an arriving proposal is compared against the store by **what it means**, not by the words it uses. A request phrased in vocabulary the entry never contains still matches when it is the same concept, and one that reuses the entry's exact words does not match when it is a different concept. Keyword matching fails in both directions here, and the false-negative direction is the expensive one: the concept returns and is re-argued while a perfectly good entry sits unread.

- **REJ-5 (A match is surfaced and re-asked, never silently applied):** a match does **not** auto-decline the arriving proposal. It is surfaced — the entry, its reason, and its accumulated requests — and the decision is put again, resolving to exactly one of three outcomes: **affirm** (the new request is appended to the entry and declined), **reconsider** (the decision has changed; REJ-8 applies), or **distinguish** (related but genuinely different; the entry does not apply and the proposal proceeds on its own merits). A store that auto-declines is a store that cannot be corrected by the traffic that most reliably shows it is wrong.

- **REJ-6 (Accumulated demand is visible, and is evidence):** each entry carries the **requests that have arrived for it** — how many, from whom or where, and when. The count and spread are not bookkeeping: independent arrivals at the same concept are the single strongest available signal that a rejection deserves reconsideration, and the reason per-request records are inadequate is that they destroy exactly this. An entry whose demand is rising is surfaced as such, not merely matched against.

- **REJ-7 (The store is read before proposing, not only before deciding):** the store is consulted by whatever **generates** proposals — a survey, a triage intake, an improvement pass — before the proposal is formed, not only by whoever decides on one already filed. Checking only at the decision point means the cost of re-derivation, re-writing and re-reading has already been paid, and the reader has already been trained to skim. (The generator-side placement `l1-negative-specification` NEG-4 requires, applied to decisions rather than to constraints.)

- **REJ-8 (Reconsideration retires the entry; closed requests stay closed):** when a decision changes, the entry is **retired** — removed from the matching set, retained only as history — and the proposal that triggered the reconsideration carries the work forward. Previously closed requests are **not** reopened: they are historical records of decisions correctly made under the reasons then in force. Reopening them manufactures a queue of items whose original reporters have moved on, and confuses the record of what was decided with the record of what is now being done.

- **REJ-9 (Proposals only; defects are never entries):** the store covers **proposals** — requests to add, change, or restructure something. A **defect** — a report that something does not do what it claims — never becomes an entry, whatever its disposition. A defect is diagnosed, fixed, or shown not to be one; a store that accepts declined defects becomes the place where broken things are argued away, and every subsequent reader has to distinguish "we decided not to build this" from "we decided this failure does not count".

- **REJ-10 (An entry is authored to be read cold):** the reason is substantive and durable enough for a stranger to **apply**, not merely to read: it names the project's scope or philosophy, a technical constraint and what it costs to lift, or a strategic choice and its alternative. "We do not want this" is not a reason; it is the decision restated. An entry that cannot be applied by someone who was not there will be re-litigated by them, which is the outcome the store exists to prevent.

- **REJ-11 (Authored on the decider's authority, never by the party that would do the work):** an entry names **who decided**, and an actor MUST NOT record a rejection of a proposal that was made *to it* on its own authority. The incentive is obvious and the failure is quiet: the party that would otherwise perform the work is the party that most benefits from the work being declined, and a store it can write to becomes a record of what it preferred not to do. (The producer-does-not-hold-the-gate rule, at the scope grain.)

- **REJ-12 (An entry is not a generator constraint until it is made one):** a rejection records that a proposal was declined; it does **not** by itself instruct any generator to avoid the concept. Promoting an entry into an exclusion that steers production is a **separate, deliberate act** under the exclusion contract, with that contract's own origin, provenance, and revisit rules. Conflating the two would silently convert every scope decision into a standing production constraint, and the exclusion set would grow with the store rather than with anyone's intent.

## 4. Detailed Design

### 4.1 What is and is not an entry

The two exclusions are the store's whole integrity, so they are worth stating as a table rather than as prose:

| Disposition | Entry? | Closed by | Why |
| --- | --- | --- | --- |
| Declined — out of scope, constrained, or a strategic choice | **Yes** | The entry, plus a reference to it | The durable case (REJ-1, REJ-10) |
| Deferred — not now, no capacity, later | **No** | The planning plane | Circumstance expires; an entry would not (REJ-2) |
| Already satisfied — it exists | **No** | A pointer to where it lives | An entry would be a false rejection (REJ-3) |
| A defect, however disposed | **No** | Diagnosis or fix | Not a proposal (REJ-9) |
| Superseded — a different proposal won | **Yes**, on the superseded concept | The entry, naming what won | The next proposer needs the comparison |

### 4.2 Entry shape

```text
entry:
    concept       := <short, recognizable name for the declined concept>
    decision      := <what was decided, in one line>
    reason        := <durable case: scope, constraint and its cost, or strategy and its alternative>
    decided_by    := <the deciding authority>          # REJ-11
    requests      := [ <arrival, with when and where> ]  # REJ-6, appended over time
    status        := active | retired(<when, why>)      # REJ-8
```

Everything except `requests` is written once and edited rarely. `requests` is the field that moves, and it is the field the reconsideration signal is read from.

### 4.3 The intake path

```text
on proposal p:
    matches := store.match_by_concept(p)              # REJ-4
    if matches is empty: proceed normally
    else:
        surface(matches, with reasons and demand)     # REJ-5, REJ-6
        outcome := decide(affirm | reconsider | distinguish)
        affirm      -> entry.requests += p ; decline p
        reconsider  -> entry.retire() ; p proceeds     # REJ-8
        distinguish -> p proceeds on its own merits
```

Two properties of this path are load-bearing. It runs at **generation** time as well (REJ-7), so a survey consults the store before writing a candidate rather than after. And no arm of it declines automatically: the store's output is always material for a decision, never the decision.

### 4.4 Reconsideration signals

An entry earns a second look on any of:

- **Rising independent demand** (REJ-6) — arrivals from unrelated sources are worth more than repeats from one.
- **A named constraint lifting** — the entry's reason cites a technical cost; that cost changed.
- **A scope change** — the project's own boundaries moved, and the entry cites them.

None of these auto-retires an entry. They raise it for the decision REJ-5 requires, which is the same decision, taken again with new evidence.

## nodus-relevance mapping

- **Declined language constructs are the clearest case.** A construct proposed and declined — because it duplicates an existing one, or because it would break a portability guarantee — is exactly a concept-keyed entry, and it is re-proposed reliably, because each new author arrives at it from the same direction.
- **The reason must survive the proposer.** A construct declined for a portability constraint needs the constraint and its cost recorded, not the verdict; a future author with a genuine case needs to argue against the constraint, which they cannot do if only the verdict was kept.
- **Demand as vocabulary evidence.** Repeated independent requests for the same missing construct are the same signal the vocabulary layer uses when a term keeps being reached for — evidence about the language's shape, not noise to be closed.

## 5. Implementation Notes

1. **Draw the REJ-3 split at intake, not at write time.** By the time an entry is being written, "declined" has already been said. The question *was this declined or was it already delivered?* belongs at the moment of disposition, where the answer is still obvious.
2. **Make `requests` append-only.** REJ-6's signal is the arrival history; an implementation that overwrites it with a count loses the spread, which is the half that distinguishes rising demand from one persistent asker.
3. **Retire, do not delete** (REJ-8). A retired entry out of the matching set but present in history answers the next question, which is invariably *did we not decide against this once?*
4. **Surface the whole entry, not a verdict** (REJ-5). A match rendered as "previously rejected" reproduces the auto-decline this forbids in the human's reading of it, even when the mechanism did not.
5. **Keep it consultable.** If the store stops being short enough to read end to end, the cause is almost always REJ-1 or REJ-2 being violated — request-grain entries, or deferrals that should have expired.

## 6. Drawbacks & Alternatives

- **A rejection store can ossify a project.** The real risk, and the reason REJ-5 forbids auto-decline and REJ-6 makes rising demand visible. The store's purpose is to make re-litigation *informed*, not to make it impossible.
- **Concept matching is a judgement.** Accepted; REJ-4 states the requirement rather than a mechanism, and REJ-5 puts every match in front of a decision, so a wrong match costs a moment rather than a wrong outcome.
- **Entries go stale as the project moves.** Real, and handled by retirement (REJ-8) plus the §4.4 signals rather than by an expiry timer — an entry whose reason still holds should not expire because time passed, and one whose reason has lapsed should not survive because it has not.
- **Alternative — rely on the closed items themselves.** Rejected: that is the current state and it produces every failure in §1. The record exists but is not where the next proposer looks, is per-request rather than per-concept, and mixes deferrals, deliveries, and defects with genuine rejections.
- **Alternative — record every declined request, not every declined concept.** Rejected by REJ-1: it scales with traffic instead of with decisions, and destroys the demand signal by scattering one concept across many rows.
- **Alternative — auto-decline on a match.** Rejected by REJ-5: it removes the only correction path the store has, and it is wrong most catastrophically in exactly the case where a concept has genuinely become right.
- **Alternative — fold into `l1-negative-specification`.** Rejected by REJ-12: an exclusion is a standing constraint injected into production and checked against output; a rejection is a decision about scope with no production side at all. Folding them would convert every declined proposal into a generator constraint, which is neither intended nor bounded.
- **Alternative — fold into the triage or planning plane.** Rejected by REJ-2 and REJ-7: the planning plane is exactly where deferrals belong and where rejections rot, and neither plane is read at *generation* time, which is where the re-proposal loop has to be broken.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[NEGATIVE]` | `.design/main/specifications/l1-negative-specification.md` | The exclusion contract an entry may be promoted into (REJ-12) |
| `[TRIAGE]` | `.design/main/specifications/l2-trigger-triage.md` | The intake path where the REJ-3 split is drawn |
| `[LEDGER]` | `.design/main/specifications/l1-operational-ledger.md` | The record plane entries live on |
| `[IMPROVE]` | `.design/main/specifications/l1-improvement-loop.md` | The generator REJ-7 requires to consult the store |
| `[FRUGALITY]` | `.design/main/specifications/l1-solution-frugality.md` | The disposition that produces the declines this preserves |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-26 | Core Team | Initial concept — the durable record of declined proposals, closing the re-proposal loop that every recurring survey and every fresh session reopens. Keyed by **concept, never by request**, so the store scales with decisions rather than with traffic (REJ-1); a **deferral is not a rejection** — circumstance expires, an entry does not, and recording one permanently closes what nobody decided against (REJ-2); **already-satisfied is not rejected** — closed by pointing at where the capability lives, since an entry there contaminates matching with a false rejection about the project's own contents (REJ-3); matched by concept, not wording, with the false-negative direction the expensive one (REJ-4); a match is **surfaced and re-asked, never auto-applied**, resolving to affirm / reconsider / distinguish (REJ-5); accumulated independent demand is carried on the entry and is the strongest reconsideration signal there is (REJ-6); the store is read at **generation** time, not only at decision time, or the re-derivation cost is already paid (REJ-7); reconsideration retires the entry while previously closed requests stay closed (REJ-8); proposals only — a declined defect is never an entry, or the store becomes where broken things are argued away (REJ-9); entries are authored to be **applied cold**, naming scope, a constraint and its cost, or a strategy and its alternative (REJ-10); authored on the decider's authority and never by the party that would otherwise do the work (REJ-11); an entry is **not** a generator constraint until deliberately promoted into one under the exclusion contract (REJ-12). Concept-only. |
