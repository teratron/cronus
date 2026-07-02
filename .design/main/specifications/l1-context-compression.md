# Context Compression

**Version:** 1.1.0
**Status:** Stable
**Layer:** concept

## Overview

A token-economy technique that re-encodes high-volume content into a denser form before it enters the model context, cutting token cost while preserving the information the agent needs — and keeping the original retrievable on demand. It is the third, distinct member of the token-economy family, sitting beside *selection* (dropping low-priority content to fit a budget) and *summarization* (lossy semantic condensation of old history). Compression is neither: it is reversible-or-fidelity-bounded dense re-encoding, applied to content that is bulky but structured — tool outputs, diffs, logs, file dumps — where most of the tokens are redundant formatting rather than meaning.

## Related Specifications

- [l2-context-management.md](l2-context-management.md) - Owns selection (trim cascade) and summarization (compaction); this concept adds the compression stage that runs before eviction.
- [l1-output-contracts.md](l1-output-contracts.md) - Structured, size-bounded results; compression is how a large result is bounded without losing recoverability.
- [l1-code-intelligence.md](l1-code-intelligence.md) - CI-6 budget-bounded context assembly with compression accounting; this concept defines the compression that accounting measures.
- [l1-tool-composition.md](l1-tool-composition.md) - Tool/nested-call outputs are the dominant compressible content.
- [l2-agent-session.md](l2-agent-session.md) - The turn loop where compression is applied before the context budget is enforced.
- [l1-cache-stable-context.md](l1-cache-stable-context.md) - Compression is a live-zone-only transform; it MUST NOT mutate the cached frozen prefix (CSC-2). Cache stability constrains *where* compression may act; this spec constrains *what* it does there.

## 1. Motivation

In a tool-heavy agent session, most tokens are not reasoning — they are the verbatim output of commands: a `git diff`, a `grep` over a repo, a stack trace, a directory listing, a JSON dump. This content is bulky but highly redundant: repeated indentation, boilerplate framing, line prefixes, whitespace, and structure the model can reconstruct. Sending it raw burns budget that could hold actual work, and pushes the session toward eviction sooner.

Selection and summarization both *lose* content — selection drops it, summarization paraphrases it away. Neither is right for a fresh `git diff` the agent is about to act on: dropping it loses the task, summarizing it loses the exact lines. What that content needs is **compression** — re-encode it densely so it costs fewer tokens but the agent can still use it (and expand it back if needed). Done with fidelity bounds, eligibility rules, and accounting, this is a large, safe token saving that the other two techniques cannot provide. Done carelessly, it silently corrupts the content the agent reasons over — so the invariants below are what make it safe.

## 2. Constraints & Assumptions

- Compression is a token-economy optimization, not a storage or transport concern; it shapes what enters the model context.
- It composes with — and runs before — the selection/summarization cascade: compress first, then trim only if still over budget.
- Eligibility is content-aware: bulky structured content is compressed; short or semantically dense prose is left alone.
- The concrete encodings are an implementation choice; this spec constrains fidelity, reversibility, eligibility, accounting, and ordering — not the algorithm.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate:

- **CC-1 (Distinct from selection and summarization):** compression re-encodes content into a denser representation; it MUST NOT be conflated with dropping content (selection) or paraphrasing it (summarization). The three are separate token-economy stages with separate guarantees.
- **CC-2 (Fidelity bound):** a compression transform is either lossless, or its information loss is explicitly declared and bounded. A transform that silently discards information the agent may need is forbidden; lossy compression of content the agent is about to act on requires an explicit eligibility decision.
- **CC-3 (Recoverable original):** the uncompressed original is retrievable on demand for any compressed content within its lifetime, so the agent (or the user) can expand it when the dense form is insufficient. Compression never destroys the only copy.
- **CC-4 (Content-aware eligibility):** only content whose token cost is dominated by redundant structure (tool outputs, logs, diffs, dumps) is eligible; semantically dense prose, code the agent is editing, and already-small content are excluded by default. Eligibility is a declared policy, not a blanket transform.
- **CC-5 (Model legibility):** compressed content presented to the model is either still directly usable by it, or accompanied by enough signal that the model knows it is compressed and how to request expansion. The model is never silently fed a lossy form it cannot tell is lossy.
- **CC-6 (Accounted):** every compression is measured — tokens before, tokens after, transform applied — and the saving is reported (the accounting CI-6 assumes). Compression that cannot be measured cannot be trusted or tuned.
- **CC-7 (Ordering before eviction):** compression runs before the selection/summarization cascade. Budget pressure is first relieved by re-encoding redundant content; only content that is still over budget after compression is subject to trimming or summarization.
- **CC-8 (Composable & idempotent-safe):** transforms may stack, but stacking MUST NOT corrupt content or double-compress unsafely; re-applying compression to already-compressed content is a no-op or a declared further reduction, never garbling.

- **CC-9 (Protected-region deny-list):** [ADDED v1.1.0] eligibility (CC-4) is overridden by a hard deny-list that is never compressed regardless of size or redundancy: (a) content the caller explicitly tags as protected, (b) safety-critical content whose exact form is load-bearing, and (c) cryptographic signatures and encrypted/redacted reasoning payloads. The deny-list is checked first; a size or redundancy heuristic can never promote a denied region into eligibility. This makes the fidelity guarantee (CC-2) unconditional for regions where any loss is unacceptable, and aligns with the sacrosanct-field rule of the cache-stability contract (`l1-cache-stable-context.md` CSC-7).

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The Token-Economy Family

| Stage | What it does | Loss | Owner |
| --- | --- | --- | --- |
| Selection / trim | drops low-priority content to fit budget | content dropped | l2-context-management |
| Summarization / compaction | condenses old history into a summary | paraphrased, lossy | l2-context-management |
| **Compression** | re-encodes bulky content densely | none, or bounded+declared | **this spec** |

### 4.2 Pipeline Position

```text
[REFERENCE]
assemble_context(messages):
    for m in messages where eligible(m):          // CC-4
        m.content := compress(m.content)           // CC-1/CC-2, record before/after (CC-6)
        keep recoverable(original)                 // CC-3
    if still_over_budget():
        trim_then_compact()                        // CC-7: selection/summarization after compression
```

### 4.3 Eligibility & Fidelity

A policy classifies content: high-volume structured output → compress (prefer lossless re-encoding); content the agent is actively editing → exclude; small content → skip. Lossy transforms are gated to content where the loss is acceptable and declared (CC-2), and the original stays recoverable (CC-3).

### 4.4 Model Interaction

When compressed content is sent, the model either reads the dense form directly or sees a marker indicating compression with a way to request the original (CC-5). Expansion-on-demand routes back to the recoverable original (CC-3).

### 4.5 Reversibility Realization — Compress-Cache-Retrieve [ADDED v1.1.0]

CC-3 (recoverable original) is realized concretely by a **compress-cache-retrieve**
mechanism rather than by keeping the original inline:

```text
[REFERENCE]
compress(content):
    dense, dropped := transform(content)          // lossy on the wire
    handle := hash(content)                         // stable content-addressed key
    store.put(handle, content)                      // stash the ORIGINAL, keyed by handle
    return dense + marker(handle)                   // CC-5: dense form carries a retrieval handle

retrieve(handle):                                   // a first-class tool the model may call
    return store.get(handle)  or  MISS              // serves the original back on demand
```

Properties that make this safe and cheap: the wire form is lossy but the end-to-end
path is lossless (the original is one `retrieve(handle)` away); the store is a
pluggable backend (in-memory for tests, durable for production) with a TTL bounding
the recoverability *lifetime* CC-3 references; a store-init failure is surfaced, never
silently downgraded to "recovery unavailable." The retrieval handle is a normal tool
call, so expansion-on-demand (CC-5) needs no special protocol — the model asks for the
original exactly as it asks for anything else. The compressed placeholder and its
handle live in the live zone only (`l1-cache-stable-context.md` CSC-2).

## 5. Drawbacks & Alternatives

- **Fidelity risk:** an over-aggressive transform can drop information the agent needed; mitigated by CC-2 (bounded, declared loss), CC-3 (recoverable original), and CC-4 (conservative eligibility). The safe default is lossless.
- **Added complexity in the hot path:** compression runs on every assembled context; justified because tool-heavy sessions are dominated by compressible content, and CC-6 accounting proves the saving is real before it is trusted.
- **Alternative — only trim and summarize:** rejected; both lose content, and neither suits fresh, must-keep bulky output (a diff the agent is about to apply). Compression keeps it usable and cheap.
- **Alternative — make it an opaque step inside context-management:** rejected; compression's fidelity/recoverability/eligibility invariants are cross-cutting (they also govern tool-output bounding and recall), so it is a shared contract, not one cascade's private detail.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[CONTEXT]` | `.design/main/specifications/l2-context-management.md` | The selection/summarization cascade this compression stage precedes. |
| `[OUTPUT]` | `.design/main/specifications/l1-output-contracts.md` | Size-bounded structured results that compression realizes recoverably. |
| `[CODE-CTX]` | `.design/main/specifications/l1-code-intelligence.md` | CI-6 compression accounting this concept supplies. |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.1.0 | 2026-07-02 | Core Team | Added CC-9 (protected-region deny-list — caller-tagged / safety-critical / crypto-thinking content never compressed regardless of size, overriding CC-4 eligibility) and §4.5 (compress-cache-retrieve realization of CC-3 — content-addressed retrieval handle + pluggable store + TTL, lossy-on-wire/lossless-end-to-end, retrieval as a normal tool call). Linked to l1-cache-stable-context (compression is a live-zone-only transform, CSC-2). |
| 1.0.0 | 2026-06-26 | Core Team | Initial spec — context compression as the third token-economy stage beside selection and summarization: reversible/fidelity-bounded dense re-encoding of high-volume content, recoverable originals, content-aware eligibility, model legibility, accounting, ordering-before-eviction, composable-safe stacking (CC-1…CC-8). |
