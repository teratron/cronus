# Remote Liveness Verdict

**Version:** 1.0.0
**Status:** RFC
**Layer:** concept

## Overview

The discipline for answering one question honestly when the answer must come from across a communication boundary: **is the remote entity still alive, or did we just lose the wire?** A local process either exists or it doesn't, checkable by asking the same machine's own process table. A remote one — behind a network link, a relay, a detached daemon on someone else's host — cannot be asked that way, and the observer's own silence, timeout, or closed socket is not an answer about the entity at all. It is an answer about the wire.

Treating "we cannot currently observe it" as "it is gone" is the single most damaging inference an execution-host abstraction can make: it orphans work that is still running, and it can cold-start a duplicate over the same resource the original never released. Treating it as "it is still fine" is the opposite failure, silently accumulating dead work nobody ever reconciles. The discipline is a **closed three-state verdict**, an evidence rule that decides which state a given signal is allowed to produce, and a small set of composing rules — what a disconnect does and does not do, what a returned status is actually a claim of, and why one physical resource must never carry two independent liveness identities at once.

## Related Specifications

- [l1-execution-locus.md](l1-execution-locus.md) — LOC declares that a locus is an addressable dimension with its own operational facts (LOC-10) and that an absent capability refuses visibly rather than falling back (LOC-6); this spec is what a locus's liveness-reporting capability must itself return once it exists, and RLV-8 is the same-physical-resource discipline LOC's per-locus substitution model assumes but does not itself enforce across two different locus *registrations* of one machine.
- [l1-work-liveness.md](l1-work-liveness.md) — WL-8 classifies a **live-but-silent** process as suspect/critical from progress staleness; it presupposes a channel exists and asks whether the work behind it is progressing. This spec covers the case WL-8's premise fails: the channel itself is gone, and the question is existence, not progress. WL-2's compare-and-clear release ("never clearing a claim a successor run already re-acquired") is the same generation-fencing principle this spec's RLV-4 applies to a liveness verdict instead of a work claim.
- [l1-crash-recovery.md](l1-crash-recovery.md) — CR-10 reclaims a **local** exclusive hold only on positive evidence the holder no longer exists, and treats an unresponsive-but-undetermined holder as never-reclaimable. RLV generalizes the same positive-evidence discipline to a **remote**, disconnection-prone channel, adding the source-of-signal test (RLV-3) a local process table never needed.
- [l1-artifact-derived-observation.md](l1-artifact-derived-observation.md) — ADO-3's three-way *absent / unreadable / empty* split is the same anti-collapse instinct applied to a different pair: reading a file left behind, not asking a live remote entity whether it still exists. Cited as the sibling pattern, not as coverage.
- [l1-tool-receipts.md](l1-tool-receipts.md) — a receipt proves an act happened; RLV-6 states the narrower thing an *artifact* proves about a *remote run* it did not itself observe end-to-end.
- [l1-foreign-agent-invocation.md](l1-foreign-agent-invocation.md) — FAI governs a one-shot **commissioned** foreign-agent run with a bounded deadline and a single verified outcome; this spec governs the different case of a **long-lived, continuously reachable** remote execution host whose liveness must be classified repeatedly over the run's life, not resolved once at the end.

## 1. Motivation

Left unspecified, an implementation improvises, and every improvisation collapses the same distinction:

- **Timeout read as death.** A socket closes, a lookup throws, a heartbeat lapses — and the observer reports the remote work as finished or gone. The work is still running; only the wire failed.
- **Correlated silence read as independent deaths.** Every entity behind one host goes quiet at the same instant. Reported one at a time, this looks like N unrelated failures instead of one transport event, and N recovery actions fire where one diagnosis was needed.
- **A stale or superseded signal reviving a settled verdict.** A termination event arrives late, for an incarnation that has already been superseded, and is applied to the current one anyway — undoing a correct verdict on account of a signal about something else.
- **A returned status trusted as a claim of what happened.** An operation reports success without having run, or reports failure after quietly succeeding, because the return value was trusted over the durable state it should have changed.
- **An artifact treated as proof of more than it shows.** A matching commit proves a commit reached the remote; it is read as proof that *this run* produced it, or that nothing later is missing, neither of which it can establish.
- **The same machine registered twice, under two different execution-host models.** Once as a directly-driven remote target, once as a self-sufficient peer with its own control plane. Every query about that machine now depends on which registration answered it, and the two silently disagree.

## 2. Constraints & Assumptions

- **The channel can fail independently of the work.** A remote execution host's process table, filesystem, and running agents are not the observer's own; the observer's only access is through a transport that can drop, stall, or lie by omission.
- **Positive evidence has an owner.** Only the host that actually holds the process, file, or record can produce evidence that it is gone. Every other party's silence is evidence about the party, not about the process.
- **The relationship between disconnection and termination is a policy, not a given.** Whether losing contact eventually terminates the work, and on what schedule, is configurable and may not be visible to whatever is asking the question right now.
- **Generations are reused.** Process identifiers, connection identifiers, and pane/session addresses get recycled; an identity check that ignores this treats a new occupant of an old identity as the old one.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **RLV-1 (A closed three-state verdict, no synonyms, no collapsing):** every liveness question about a remote entity resolves to exactly one of **alive** / **unverifiable** / **confirmed-gone** — never a boolean, never a fourth state, and never a field that quietly means one of these under a different name. **Unverifiable** MUST NOT be collapsed into either neighbour: it is not "probably alive" and not "probably gone," it is the honest statement that the question could not be answered this time. An implementation that has no field for *unverifiable* has not solved this problem; it has hidden it inside whichever of the other two states absorbs the ambiguous case.

- **RLV-2 (Confirmed-gone requires evidence sourced from the owning host):** a **confirmed-gone** verdict requires **positive evidence from the party that actually holds the resource** — the host reporting its own process exited, its own file deleted, its own record cleared. Any signal that did not originate there — a timeout, a closed socket, absence from the observer's local bookkeeping, a lookup that threw, a request that never returned — is **unverifiable by construction**, regardless of what field it is written into or how confidently the code that produced it is named. This is the single rule the rest of the discipline exists to protect: the failure mode it prevents is a transport problem masquerading as a fact about the world.

- **RLV-3 (Correlated silence is diagnosed as the channel, not multiplied into independent deaths):** when every entity known to be behind one channel — one host, one connection, one relay — goes silent at once, that simultaneity is itself evidence, and it points at the **channel**, not at each entity individually. A verdict engine MUST test for this before promoting any one entity's silence toward confirmed-gone: a channel-level cause diagnosed once is correct; the same silence read per-entity multiplies one transport event into N false individual verdicts and N unwarranted recovery actions.

- **RLV-4 (A termination signal is checked against the current incarnation before it is trusted):** a signal reporting termination is accepted as evidence for the entity's **current** incarnation, generation, or identity only when the signal names that current incarnation. A signal for a superseded incarnation, a stale generation, or an identity that has since been reassigned MUST NOT be applied to whatever currently holds that name. This is WL-2's compare-and-clear discipline (never clearing a claim a successor run already re-acquired) generalized from a work-claim to a liveness verdict: the entity behind an address changes over time, and a verdict is only ever a verdict about who was there when the evidence was produced.

- **RLV-5 (A returned status is not self-certifying):** whether an operation's reported outcome is trusted is decided by checking the **durable state that operation was supposed to change**, not by trusting its return value alone. A report of success does not establish that anything ran; a report of failure does not establish that nothing succeeded. Composes AO-6 (a supplied figure is not its own proof) and CMP-1 (fresh evidence this turn) at the remote-liveness grain specifically.

- **RLV-6 (Artifact evidence proves what it names and nothing wider):** a durable artifact found on or through the remote side — a pushed commit, a written file, a matched record — is **stronger** evidence than a liveness signal, but it proves a **narrower** claim than it appears to. It proves that the artifact exists at the location checked; it does not by itself establish that the run currently being asked about produced it, that no later work is missing, or that the search covered every place the answer could be. An empty result proves the search found nothing at its declared scope, not that nothing exists outside it. A verdict built on artifact evidence names the scope it actually checked.

- **RLV-7 (Disconnection is not, by default, termination — and any configured link between them is disclosed with its terms):** where the platform's design intent is that remote work outlives a client disconnecting, a lost connection alone MUST NOT be reported as confirmed-gone, and MUST NOT trigger termination as a side effect of the observer merely leaving. Where a bounded grace period or an explicit user action *can* eventually end the work, the verdict-producing surface states which mechanism is in force and, where the schedule is not directly observable from the querying side, defaults to **unverifiable** rather than guessing which branch of the policy currently applies. A disconnect that merely drops the observer's ability to *command* the remote entity (its control plane) is distinguished from a disconnect that ends the entity itself (its data plane / actual execution) — losing the former is never evidence about the latter.

- **RLV-8 (One physical resource carries exactly one liveness identity at a time):** where a physical or logical resource can be reached through more than one execution-host abstraction — driven directly versus registered as a self-sufficient peer with its own control plane, for instance — it is registered under **exactly one** identity at a time, chosen deliberately, never both concurrently. Registering the same resource under two identities is not redundancy; it splits its inventory, so a query through one identity and a query through the other disagree about what that resource is doing, and neither disagreement is wrong given what each identity alone can see. This composes LOC's locus-substitution model (a locus is the unit of substitution) by adding the rule LOC assumes but does not itself state: two live locus registrations MUST NOT name the same underlying resource.

> An L2 implementation cannot reach RFC until every invariant above is addressed in its Invariant Compliance section.

## 4. Detailed Design

### 4.1 The verdict machine

```
                 signal arrives
                       |
        was it sourced from the owning host? ── no ──► unverifiable (RLV-2)
                       | yes
        does it match the current incarnation? ── no ──► unverifiable (RLV-4)
                       | yes
        did every sibling on the channel go quiet at once? ── yes ──► diagnose channel, not entity (RLV-3)
                       | no
                 confirmed-gone
```

A verdict that cannot pass every gate above stays **unverifiable**; nothing is promoted to confirmed-gone by default or by elapsed time alone.

### 4.2 Why the three-state vocabulary resists erosion

A two-state vocabulary always has a home for the ambiguous case, and that home is wherever the implementation finds it more convenient to write to — usually "alive," because reporting gone is the more consequential mistake and engineers correctly fear it, or occasionally "gone," because a caller wants a clean boolean to branch on. Both erosions are silent: nothing crashes, nothing errors, the code simply stops distinguishing the case the third state exists to hold. The state must be **structurally present** — a field, a variant, a return type — that a type system or a schema can refuse to let a caller skip past, not a comment saying "treat timeouts carefully."

### 4.3 Composing with work-claim liveness

This spec and WL-8 answer different questions that are easy to conflate because both produce a classification from silence. WL-8 presupposes the channel is intact and asks whether the work behind it is *progressing*. RLV presupposes the channel might not be intact and asks whether the entity *exists at all*. A system exercising both runs RLV first where the channel is in question at all — a live-but-silent verdict from WL-8 is only meaningful once RLV has established the channel is actually delivering signal.

### 4.4 Failure modes named

| Failure | What it looks like | Which invariant closes it |
| --- | --- | --- |
| Timeout read as death | Live work orphaned, or a duplicate cold-started over it | **RLV-2** |
| N false individual deaths from one dropped link | A recovery storm where one diagnosis was needed | **RLV-3** |
| Stale signal resurrecting a settled verdict | A correct "gone" undone by evidence about a different incarnation | **RLV-4** |
| Trusting the return value | A "succeeded" that never ran, or a "failed" that quietly did | RLV-5 |
| Over-reading an artifact | A matched commit treated as proof the current run is complete | RLV-6 |
| Disconnect treated as termination | Work killed by the observer merely leaving | **RLV-7** |
| Split inventory | The same machine answers differently depending on which registration is asked | **RLV-8** |

## 5. Implementation Notes

1. **Name the states in the type system, not in a comment.** A verdict type with exactly three variants, none nullable into meaning a fourth thing, is what keeps RLV-1 from eroding under deadline pressure.
2. **Log which gate produced the verdict**, not only the verdict itself — RLV-2/3/4's ordering is diagnostic value that a bare `unverifiable` throws away.
3. **Treat the channel-level diagnosis (RLV-3) as its own event**, separate from the per-entity verdicts it explains, so an operator sees one transport incident rather than a flood of individually-unremarkable silences.
4. **Bind incarnation-matching (RLV-4) to whatever identity scheme the platform already uses for the resource** — a process-identifier-plus-fingerprint pair, a monotonic generation counter, a connection epoch — rather than inventing a parallel one.

## 6. Drawbacks & Alternatives

- **RLV-1 makes the common case slightly more verbose.** Most callers want a boolean. The discipline exists precisely because that convenience is where the erosion in §4.2 starts.
- **RLV-7's conservative default costs a slower recovery from a genuinely dead remote host.** Accepted: the cost of waiting a little longer on a truly-dead entity is bounded and visible; the cost of declaring a live one dead is a duplicate or a data loss that surfaces much later and is harder to trace back.
- **Alternative — infer liveness from title strings, log tails, or other heuristic surfaces.** Rejected as the generative case of every failure in §1: a heuristic surface was never designed to answer this question and does not carry the positive-evidence property RLV-2 requires.
- **Alternative — fold this into `l1-work-liveness`.** Rejected: WL's subject is progress classification of work whose channel is presumed intact (§4.3); this spec's subject is existence classification when the channel itself is the open question, which is a different precondition and a different evidence discipline (source-of-signal, RLV-2) that WL-8 has no analog of.
- **Alternative — fold this into `l1-execution-locus`.** Rejected: LOC declares what capabilities a locus offers and refuses absent ones (LOC-6); it does not classify whether a live capability's remote counterpart has died mid-session. RLV-8 is the one rule this spec adds *to* LOC's model rather than restating.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[LOCUS]` | `.design/main/specifications/l1-execution-locus.md` | The addressable-dimension model RLV-8 extends |
| `[LIVENESS]` | `.design/main/specifications/l1-work-liveness.md` | WL-2/WL-8, the work-claim-grain siblings demarcated in §4.3 |
| `[CRASH]` | `.design/main/specifications/l1-crash-recovery.md` | CR-10, the local-holder analog of RLV-2's evidence rule |
| `[OBSERVE]` | `.design/main/specifications/l1-artifact-derived-observation.md` | ADO-3, the sibling anti-collapse pattern for artifact reads |
| `[FOREIGN]` | `.design/main/specifications/l1-foreign-agent-invocation.md` | The one-shot commissioned-execution case this spec's continuous case is distinguished from |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-09-01 | Core Team | Initial concept — the discipline for classifying a remote entity's existence when the observer's only access is a communication channel that can fail independently of the entity itself. A closed three-state verdict — alive / unverifiable / confirmed-gone — with **unverifiable** structurally distinct and never collapsible into either neighbour (RLV-1); confirmed-gone requires positive evidence sourced from the party that actually holds the resource, so a timeout, closed socket, or local lookup miss is unverifiable by construction regardless of field naming (RLV-2); correlated silence across every entity behind one channel is diagnosed once, at the channel, rather than multiplied into independent per-entity deaths (RLV-3); a termination signal is trusted only when it names the entity's current incarnation, generalizing WL-2's claim-release compare-and-clear from work ownership to liveness verdicts (RLV-4); a returned status is checked against durable state rather than trusted on its own report (RLV-5); artifact evidence is stronger than a liveness signal but proves only the narrower claim it actually names (RLV-6); disconnection is not by default termination, the applicable policy and its schedule are disclosed rather than assumed, and losing the control plane is distinguished from losing the underlying execution (RLV-7); and one physical resource is registered under exactly one liveness identity at a time, since two concurrent registrations split its inventory and produce disagreeing answers that are each locally correct (RLV-8). Distilled from an adoption pass over an external desktop multi-agent orchestrator's SSH/remote-execution documentation. Concept-only. |
