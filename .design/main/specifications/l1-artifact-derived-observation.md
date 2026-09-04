# Artifact-Derived Observation

**Version:** 1.1.0
**Status:** Stable
**Layer:** concept

## Overview

Some of what the office most needs to know about is produced by tools it does **not**
control and cannot instrument: an external agent it delegates work to, a coding tool the
human runs beside it, a build system, a third-party runtime. The reflex is to wrap them —
a proxy, a shim, an interception layer that sees every call. That buys fidelity, and it
buys it by inserting the observer into the observed tool's critical path: from then on,
when the wrapper breaks, the user's tool breaks.

There is a second way, and it is usually the right one. Nearly every such tool already
leaves a durable trail — a session transcript, a log, a local database, a state file —
written for its **own** purposes and needing no cooperation from anyone. Observing from
that trail is **out-of-band by construction**: the observed tool cannot be slowed, cannot
be broken, does not need to agree, and does not even need to know.

What the approach costs is honesty work. The artifacts are private formats that change
without notice; the same real event often appears in two of them; a source may be absent
entirely because the tool is not installed. Each of those failure shapes has a
characteristic wrong answer — a confident zero, a silent double-count, a stale conclusion
— and the whole value of the discipline is that it names them instead of producing them.

This is the **uninstrumented-source** sibling of telemetry: telemetry is what a system
emits about itself by design; this is what an observer reconstructs about a system that
emits nothing for it.

## Related Specifications

- [l1-telemetry.md](l1-telemetry.md) — the **instrumented** counterpart, and a deliberate boundary: telemetry is what the product emits about *itself* under a consent gate; this concept covers what is reconstructed about a *foreign* tool from artifacts it wrote for its own reasons. Neither substitutes for the other, and derived observations follow the same egress gate.
- [l1-interception-model.md](l1-interception-model.md) — the **opposite** placement, stated as a contrast: an interceptor sits *on* an effect and may guard it (and can therefore break it); an artifact reader sits nowhere near the effect and can only read what it left behind. ADO-1 is the deliberate choice of the second when the tool is not ours.
- [l1-context-provenance.md](l1-context-provenance.md) / [l1-provenance-taint.md](l1-provenance-taint.md) — a foreign artifact is **untrusted content** (ADO-9): a transcript written by another agent can carry an injected instruction, and it is neutralized at the boundary exactly like any other untrusted input.
- [l1-practice-analytics.md](l1-practice-analytics.md) — the normalized-trace model and honest data-gap accounting (PA-6/PA-7) are the analysis-side expression of ADO-3 and ADO-11; this spec governs how the trace is *obtained* when nobody emitted one.
- [l1-outcome-attributed-cost.md](l1-outcome-attributed-cost.md) — the most demanding consumer: attributing spend to outcomes across tools the office did not run requires exactly this acquisition discipline, and inherits its coverage honesty.
- [l1-agent-federation.md](l1-agent-federation.md) — the **cooperating-peer** case, and its complement: a federated peer has an identity and a protocol; an artifact-observed tool has neither, and the two must not be conflated.
- [l1-progressive-disclosure.md](l1-progressive-disclosure.md) — PD's lazy-expansion discipline is what ADO-6 applies to adapters: breadth of supported sources costs nothing until a source is actually present.
- [l1-anomaly-consensus.md](l1-anomaly-consensus.md) / [l1-observation-retention.md](l1-observation-retention.md) — downstream consumers of derived records; OR-7's resolution-and-truncation honesty is the retention-side parallel of ADO-11 coverage disclosure.
- [l1-security.md](l1-security.md) — the artifacts are the user's own work product: on-device by default, never copied out without a visible reason, and any secret encountered is never re-emitted (ADO-8).
- [l1-workflow-language.md](l1-workflow-language.md) — the nodus projection (§4.6): value provenance and origin taint already carry everything needed to mark artifact-derived values as derived **and** untrusted; no new primitive.

## 1. Motivation

**A wrapper makes the observer a liability for the observed.** The moment observation
requires the tool to run through something the office supplies, every defect in that
something is a defect in the user's tool. The out-of-band reading has strictly weaker
fidelity and strictly better failure behaviour: the worst outcome of a broken reader is a
missing number, not a broken workflow.

**The characteristic failure of this approach is a confident zero.** A source that is
absent, unreadable, or newly reformatted produces "no records", which aggregates
indistinguishably from "nothing happened". The number that results is not merely wrong —
it is wrong in the direction that makes everything look fine, and it will be read as a
fact about the world rather than about the reader.

**The same event shows up more than once.** Tools mirror each other's sessions, hosts keep
their own copy, exports duplicate what the log already had. Naïve aggregation over "all
sources" double-counts, and the resulting overstatement is invisible without an event
identity that survives crossing between sources.

**Private formats change without notice, because they were never a contract.** The
artifact belongs to the tool; nobody promised its shape, and no version negotiation
exists. A reader that assumes stability produces silent misparses after an upgrade — the
worst kind, because the pipeline still returns well-formed numbers.

**Supporting many tools must not tax the user who has one.** Breadth is the whole point of
this approach, and breadth implemented eagerly means every user pays for every adapter's
dependencies. The cost model has to make an unused adapter genuinely free.

## 2. Constraints & Assumptions

- The observed tool is a **black box**: no API contract, no cooperation, no notification of
  format change. The only interface is what it happens to write.
- The artifacts are **local** to the user's machine and belong to the user. Nothing here
  authorizes network collection.
- Observation is **read-only**: the observer never writes, cleans, compacts, rotates, or
  "repairs" an artifact it did not create.
- The concept covers **acquisition and normalization** only. What is computed from the
  resulting records — analytics, detection, cost attribution — belongs to the consumers.
- Fidelity is **best-effort by definition**. Every consumer of these records is assumed to
  need the coverage figure alongside the number.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **ADO-1 Out-of-band by construction**: observation reads artifacts the tool writes for
  its **own** reasons. The observer MUST NOT wrap, proxy, shim, inject into, or otherwise
  become a dependency in the observed tool's execution path, and MUST NOT modify the
  artifacts it reads. The guarantee bought by this restriction — the observer's failure
  can never become the observed tool's failure — is the reason the approach is chosen over
  a higher-fidelity interception, and it is void the moment anything is inserted into that
  path.

- **ADO-2 The format is a discovered shape, never a promised contract**: each reader
  **declares the shape it expects** and validates before interpreting. A shape mismatch is
  a typed **coverage gap for that source**, reported as such — never an exception that
  fails the whole observation, and never a silent reinterpretation that yields plausible
  numbers from an unrecognized structure. The artifact's owner owes no compatibility and
  will break it; the reader's job is to notice.

- **ADO-3 Absent, unreadable, and empty are three different answers**: *the tool is not
  present*, *the artifact exists but could not be read or parsed*, and *the artifact is
  present and genuinely records nothing* are distinguished and reported separately.
  Collapsing them produces the characteristic failure of this whole approach — a confident
  zero that reads as a fact about the world.

- **ADO-4 One adapter per source, failing independently**: each observed tool has its own
  adapter — discover its sources, parse them into the shared record shape — registered
  independently. An adapter that is missing, crashes, or lacks its dependency **excludes
  its source and is reported**; it never aborts the other adapters and never silently
  reduces the reported scope.

- **ADO-5 Discovery is declared, never a filesystem sweep**: an adapter names the specific
  locations it reads. The observer does not scan the user's disk looking for things that
  might be sessions — an undeclared search reads material nobody agreed to expose and makes
  the observer's own footprint unbounded and unauditable.

- **ADO-6 Adapters load only when their source is present**: an adapter and its heavy
  dependencies are loaded lazily, on evidence that its tool is actually installed.
  Supporting N tools MUST NOT impose N tools' worth of cost, startup time, or dependency
  surface on a user who has one.

- **ADO-7 One underlying event counted once across overlapping sources**: records carry a
  **derived event identity** that is stable when the same real event appears in more than
  one artifact (a tool's own log, a host's mirror, an export), and aggregation counts such
  an event **once**. Where identity cannot be established, the possible double-count is
  **disclosed** — never quietly accepted (inflating every total) and never quietly dropped
  (deleting real events on a guess).

- **ADO-8 Incremental by fingerprint, with an explicit interpretation version**:
  observation is incremental — an unchanged artifact is not re-read, keyed on a cheap
  fingerprint. Derived results carry the **version of the interpretation that produced
  them**; changing how an artifact is read bumps that version and forces recomputation, so
  a corrected reader can never leave stale conclusions standing behind an unchanged input.

- **ADO-9 Foreign artifact content is untrusted input**: text read from an artifact another
  tool wrote — a transcript, a prompt log, a stored message — MAY contain instructions
  aimed at whatever reads it. It is treated as untrusted content and neutralized at the
  boundary before it can influence any agent decision, exactly like any other untrusted
  input. That it came from a file on the user's own disk is not provenance; it is a
  location.

- **ADO-10 Derived, never authoritative; on-device; secret-safe**: everything produced here
  is a **derivation**, superseded by the observed tool's own account wherever one exists,
  and any inferred or estimated field is labeled as such. The artifacts are the user's work
  product: content stays on-device by default, anything derived from it egresses only
  through the existing consent gate, and a secret encountered while reading is never
  re-emitted into a derived record, log, or report.

- **ADO-11 Coverage travels with every number**: which sources were covered, at what
  fidelity, and what fraction of the observed period each actually accounts for is part of
  the readout — not a footnote and not an optional detail view. A figure computed from
  partial coverage and presented as a total is the defining dishonesty of artifact-derived
  observation, and the coverage figure is what makes the number safe to use.

- **ADO-12 A measured figure expires with the window it describes, not on a clock**: [ADDED
  v1.1.0] where an observation describes a **bounded period** — a quota consumed within an
  allowance window, a rate over an interval, a count since a reset — its retained value stays
  valid until **that period ends**, and is invalidated by the period rolling over rather than
  by an elapsed-time heuristic. A cached figure outlives the probe that produced it; what it
  must not outlive is the window it is *about*, because once the window has reset the figure
  describes a period that is over and reports a consumption that is now untouched. Two
  guards keep this from degrading into data loss. A record whose **window metadata is absent
  or unparseable is kept, not discarded** — an unreadable boundary is a reason to mark the
  figure's validity unknown, never a reason to throw away a real measurement. And a
  **re-measurement interval is a defence against repetition, not against people**: a short
  reuse window exists to absorb a surface being opened and closed repeatedly, so an explicit
  request for a fresh figure bypasses it entirely, while an automatic refresh does not.
  Where a fresh measurement cannot be taken at all, the last figure whose window has not
  since reset remains the answer of record, carried with the reason the refresh failed
  (ADO-3's honest-absence discipline at the freshness grain rather than the presence grain).

- **ADO-13 A source that cannot identify events contributes a bound, never a value**: [ADDED
  v1.1.0] ADO-7 counts one real event once across overlapping sources by **derived event
  identity**. Some sources carry no identity at all — an aggregate counter, a total with no
  per-item detail, a summary written by a tool that kept nothing else. Such a source is
  neither summed into the total (which double-counts every event another source already
  identified) nor discarded (which deletes real activity nobody else recorded): it is
  admitted as a **lower bound on the answer**, reported as a bound, and never presented as a
  measurement. Where two sources overlap in time, what merges is the **set of identified
  events**, never the counts.

- **ADO-14 An authoritative figure may sit beside a derived one; neither absorbs the other**:
  [ADDED v1.1.0] ADO-10 makes everything this layer produces **derived**. Where the observed
  tool's own provider can be asked directly for the same quantity, that answer is
  **authoritative** and the two live in one record as **separately-labelled figures with
  their own provenance and their own freshness** — a derived figure never becomes
  authoritative by agreeing with one, and an authoritative figure is never back-filled from a
  derived one when it cannot be obtained. Reaching the provider is an **egress act** with the
  narrowest possible surface: a credential read for it travels nowhere but the request that
  needs it, and only an explicitly display-safe field of the result may enter the record
  (composing EA-7 and the confidentiality-flow sink discipline). A provider that cannot be
  reached leaves the authoritative side **absent and labelled absent** (ADO-3), never
  substituted.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The acquisition pipeline

```text
[REFERENCE]
observe(period):
    records := []
    gaps    := []
    for adapter in registered_adapters:
        if not adapter.source_present():      gaps += (adapter, absent);      continue   // ADO-3
        if not adapter.loadable():            gaps += (adapter, unavailable); continue   // ADO-4/6
        for source in adapter.discover():                                                 // ADO-5 declared
            if unchanged(source.fingerprint): records += cached(source);      continue   // ADO-8
            r := adapter.parse(source)
            if r is ShapeMismatch:            gaps += (source, unrecognized); continue   // ADO-2
            records += r
    return dedup(records), gaps, coverage(records, gaps, period)                          // ADO-7, ADO-11
```

Note what the pipeline never does: it never returns records without gaps and coverage
beside them. The three are one result, because a consumer that receives only the first is
structurally unable to tell a quiet period from a broken reader.

### 4.2 The three absences (ADO-3)

| Answer | Means | What a consumer should do |
| --- | --- | --- |
| **absent** | the tool is not installed / never ran | exclude from the denominator entirely |
| **unavailable** | present but unreadable, unparseable, or newly reshaped | report a gap; the period is *unknown*, not empty |
| **empty** | read successfully; the tool genuinely did nothing | count as a real zero |

Only the third is a zero. The first two are the absence of information, and a readout that
renders them as zero has converted "we do not know" into "nothing happened" — a conversion
that is invisible downstream and never recoverable.

### 4.3 Cross-source identity (ADO-7)

```text
[REFERENCE]
event_identity(record) := stable_hash(
    originating_tool_or_session, logical_timestamp, actor, operation, payload_digest)
```

The identity is **derived**, because no shared identifier exists across independently
authored tools — that is precisely the situation. Its quality determines whether
overlapping sources merge or double-count, so a weak identity is itself reported: an
aggregate whose dedup confidence is low is labeled, not silently trusted.

Dropping suspected duplicates on a weak identity is the worse error of the two available:
inflation is visible and correctable, deletion is neither.

### 4.4 Why not a wrapper

| | Interception / proxy | Artifact reading |
| --- | --- | --- |
| Fidelity | high, complete, real-time | partial, lagging, format-dependent |
| Cooperation needed | the tool must be launched through it | none |
| Failure blast radius | **the observed tool breaks** | a number is missing |
| Format risk | owns its own shape | inherits someone else's private shape |

The trade is deliberate and directional: for a tool the office **owns**, instrument it —
telemetry and the interception model exist for that. For a tool it does **not** own,
accept lower fidelity to keep the user's tooling out of the blast radius. Choosing the
wrapper for a foreign tool is choosing to be a cause of that tool's outages.

### 4.5 Boundary with the consumers

Acquisition ends at a normalized record plus its gaps and coverage. Detection, scoring,
cost attribution, anomaly ranking, and any verdict about what the records *mean* belong to
their own layers, which consume the coverage figure as a first-class input — an analysis
run over 40% coverage is a different claim from the same analysis over 95%, and only the
consumer can decide whether that is enough for the question being asked.

### 4.6 nodus projection

No new language primitive is required; the workflow layer already holds both halves:

1. **Reading is a host effect.** Discovery and parsing live behind the host provider
   surface, and the read is an ordinary declared effect class — read-only, local — so a
   host that supplies no reader simply observes nothing (additive by construction).
2. **Provenance carries "derived", origin taint carries "untrusted".** A value obtained
   from a foreign artifact is *both* a derivation (ADO-10) and untrusted-origin content
   (ADO-9), and the language already carries the two labels independently: the
   derived-ness rides the value's provenance, the untrustedness rides its origin taint, and
   the taint is what prevents an artifact's text from steering a workflow through a
   model-facing interpolation.
3. **Gaps are values, not exceptions.** A source that is absent or unreadable yields a
   typed result the workflow can branch on, consistent with the runtime's
   degrade-don't-throw posture — an unreadable source must not abort a run that has other
   sources to read.

## 5. Implementation Notes

1. Keep the coverage computation in the same pass as the records (§4.1) — computing it
   afterwards from a separate walk invites the two to disagree, and the disagreement will
   favour whichever number looks better.
2. Fingerprint on the cheapest fields that actually change (size and modification instant
   at minimum); a content hash is correct but re-reads the artifact, defeating the purpose.
3. Version the *interpretation*, not just the cache format (ADO-8): the expensive mistake
   is a corrected parser whose old conclusions survive because the input did not change.
4. Treat "five adapters have no test coverage" as a coverage gap in the ADO-11 sense, not a
   backlog item: an untested reader's silence is indistinguishable from a quiet tool.

## 6. Drawbacks & Alternatives

- **Fidelity is genuinely lower than instrumentation.** Accepted and central: §4.4 states
  the trade explicitly, and ADO-11 keeps the resulting numbers honest rather than pretending
  the gap is not there.
- **Readers break when foreign formats change.** Unavoidable, and ADO-2 converts it from a
  silent corruption into a reported gap. The maintenance cost is real and is the price of
  observing things nobody promised to keep stable.
- **A derived event identity can be wrong in both directions.** Bounded by ADO-7's
  asymmetry: disclose a suspected double-count rather than delete a possibly-real event,
  because inflation is visible and deletion is not.
- **Alternative — wrap or proxy the tools.** Rejected for foreign tools by ADO-1: it makes
  the office a cause of the user's tool failing. Entirely appropriate for tools the office
  owns, where telemetry and interception already apply.
- **Alternative — ask the tools to emit a standard format.** Rejected as a *dependency*:
  it requires cooperation from every vendor and yields nothing for the tools that decline.
  Where a tool does emit something standard, ADO-10's "the tool's own account supersedes"
  already prefers it.
- **Alternative — scan the disk for anything session-shaped.** Rejected by ADO-5: it reads
  material nobody agreed to expose and makes the observer's footprint unauditable.
- **Alternative — skip coverage reporting and just show the number.** Rejected by ADO-11:
  it is the one shortcut that makes every downstream conclusion unsound, and it is the
  default outcome unless the spec forbids it.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[TELEMETRY]` | `.design/main/specifications/l1-telemetry.md` | The instrumented counterpart and the egress gate derived data follows. |
| `[INTERCEPT]` | `.design/main/specifications/l1-interception-model.md` | The in-path alternative ADO-1 deliberately declines for foreign tools. |
| `[PROVENANCE]` | `.design/main/specifications/l1-context-provenance.md` | Why a foreign artifact's text is untrusted input (ADO-9). |
| `[ANALYTICS]` | `.design/main/specifications/l1-practice-analytics.md` | The consumer whose honest-data-gap accounting mirrors ADO-3/ADO-11. |
| `[OUTCOME-COST]` | `.design/main/specifications/l1-outcome-attributed-cost.md` | The most demanding consumer of these records. |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | On-device default and secret-safety for read artifacts (ADO-10). |
| `[WORKFLOW-LANG]` | `.design/main/specifications/l1-workflow-language.md` | The nodus surface the discipline projects onto (§4.6). |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-05 | Core Team | Initial spec — artifact-derived observation as the uninstrumented-source sibling of telemetry: how the office learns about a tool it does not control, by reading the durable trail that tool already writes for its own purposes. Out-of-band by construction, never wrapping or proxying, because the guarantee bought is that the observer's failure cannot become the observed tool's failure (ADO-1); the artifact's shape is discovered, never promised, so a mismatch is a typed per-source coverage gap rather than a run-ending error or a silent reinterpretation (ADO-2); absent / unreadable / empty are three different answers and collapsing them yields the characteristic failure — a confident zero read as a fact about the world (ADO-3); one independently-failing adapter per source (ADO-4); discovery declared, never a disk sweep that reads what nobody agreed to expose (ADO-5); adapters loaded only on evidence their tool exists, so breadth is free to the user who has one (ADO-6); one underlying event counted once across overlapping sources via a derived identity, with an unresolvable case disclosed rather than quietly inflated or quietly deleted (ADO-7); incremental by fingerprint with a versioned *interpretation* so a corrected reader cannot leave stale conclusions standing (ADO-8); foreign artifact text treated as untrusted input, since a file on the user's own disk is a location and not a provenance (ADO-9); derived never authoritative, on-device, secret-safe (ADO-10); and coverage traveling with every number, the disclosure that makes a partial figure safe to use (ADO-11). Nodus projection needs no new primitive — reading is a declared host effect, and the existing value-provenance and origin-taint labels already carry *derived* and *untrusted* independently. Concept-only. |
| 1.1.0 | 2026-09-04 | Core Team | Amended — ADO-12/13/14, from an external agent-usage collector that folds several tools' local artifacts and one provider's own endpoint into a single displayed record. **ADO-12**: a figure describing a bounded period expires when *that period* rolls over, not on an elapsed-time heuristic — a cached percentage outliving its allowance window reports a consumption against a window that has since reset and is now untouched; guarded both ways, since unreadable window metadata marks validity unknown rather than discarding a real measurement, and a short re-measurement interval defends against a surface being opened repeatedly, never against a person explicitly asking for fresh numbers (an explicit request bypasses it, an automatic refresh does not). **ADO-13**: a source carrying no per-event identity (an aggregate counter, a bare total) is neither summed into the answer (double-counting what another source already identified) nor discarded (deleting real activity) — it is admitted as a **lower bound**, reported as a bound; overlapping sources merge their *identified event sets*, never their counts. **ADO-14**: where the observed tool's provider can be asked directly, that answer is authoritative and sits beside the derived one as a separately-labelled figure with its own provenance and freshness — a derived figure never becomes authoritative by agreeing with one, an unreachable provider leaves the authoritative side labelled absent rather than substituted, and the credential read to reach it travels nowhere but that request, with only an explicitly display-safe field entering the record. |
