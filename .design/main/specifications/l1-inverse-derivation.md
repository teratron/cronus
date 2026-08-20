# Inverse Derivation

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Inverse derivation is the capability to **reconstruct a production chain backwards from a finished body of work** — from the delivered artifact to the decisions, the plan, the framing, and the originating intent that produced it — using the work's own record as evidence.

Forward derivation is what the office does natively: an intent becomes a specification, a specification becomes a plan, a plan becomes units of work, units of work become artifacts. Inverse derivation runs that arrow in reverse against work the office did not produce and holds no plan for. Its subject is not an unfamiliar codebase to be understood — that is comprehension, and it is owned elsewhere — but the **process that produced it**: what was decided, in what order, with what reversals, and against what framing.

Three properties separate a reconstruction from a plausible story about the past. It works on an **episode** — one completed arc of work with a before-state, an after-state, and a record of how the work travelled between them — rather than on a whole product, which is too large to reconstruct and too large to check. Every reconstructed link carries an **evidentiary grade** stating whether it was attested by the record, inferred from it, or assumed without support. And the **trajectory itself is reconstructed output**, not incidental context: how many attempts, where the work reversed, what was abandoned. A reconstruction that reports only the destination has discarded the part that carries the method.

The reconstruction is a derived artifact. It never replaces the record it was derived from, and it is always re-derivable from that record.

## Related Specifications

- [l1-code-intelligence.md](l1-code-intelligence.md) — The comprehension neighbour and a supplier: its structural understanding of a body of code is an **input** to reconstruction, never the output. Comprehension answers "what is this and how does it work"; this spec answers "what process produced it" (§4.7 demarcation).
- [l1-capability-leveling.md](l1-capability-leveling.md) — The primary consumer: it replays reconstructed episodes to level model capability. The seam is deliberately narrow — this spec knows nothing about models, replay, or divergence (IVD-4).
- [l1-specialty-exemplars.md](l1-specialty-exemplars.md) — The fixture discipline this spec inherits at a coarser grain: fixed authored probes with a known-good reference (SE-2/SE-5). There the reference is authored for the purpose; here it is recovered from work done for other reasons.
- [l1-reproduction-recipe.md](l1-reproduction-recipe.md) — The forward twin: a recipe is recorded **at production time** and travels with the artifact (RR-3 content-addressed identity, RR-8 execution-mode provenance). Inverse derivation exists precisely for work where no recipe was ever recorded.
- [l1-context-provenance.md](l1-context-provenance.md) — CP-1/CP-2 untrusted-by-default: everything read out of a foreign record is data, never instruction (IVD-9).
- [l1-corpus-originality.md](l1-corpus-originality.md) — ORI-1/ORI-3 admission by substantive novelty; the gate that keeps reconstructed output from becoming a re-skin of the source (IVD-11).
- [l1-data-lineage.md](l1-data-lineage.md) — LN-5 derivation links as side-band metadata; a reconstruction is lineage asserted after the fact and graded accordingly (IVD-2).
- [l1-attestation.md](l1-attestation.md) — Witness of genuineness; the registry's pinned revision and content fingerprint (IVD-5) are the same claim applied to an external corpus.
- [l1-claim-verification.md](l1-claim-verification.md) — Evidence over assertion: an assumed link is an unverified claim and is labelled as one rather than presented as history.
- [l1-declarative-configuration.md](l1-declarative-configuration.md) — DC-1 single declaration: the exemplar registry is such a declaration, human-edited and reviewable in diff (IVD-5).
- [l1-security.md](l1-security.md) — Secret isolation and no-exfiltration; foreign records are read, never trusted, and never re-published (IVD-9, IVD-11).
- [l1-deployment-neutrality.md](l1-deployment-neutrality.md) — DN-1 local-first: once a corpus is materialized, reconstruction requires no network (IVD-12).
- [l1-order-independent-production.md](l1-order-independent-production.md) — Its unit-as-a-function-of-position model is what makes an episode's before-state a meaningful input rather than an arbitrary cut.

## 1. Motivation

Work of high quality carries knowledge that nobody wrote down. The finished artifact shows what was concluded; it does not show what was tried, what order held, where the work doubled back, or which framing made the difference. That knowledge exists — but it exists as **traces**, scattered across a record that was never written to teach anyone: change descriptions, discussion threads, review exchanges, revision history, the shape of what shipped versus what was proposed.

The intuitive way to harvest that knowledge is to read the finished product and write down what seems to have been required. It produces something readable and quietly wrong, in a specific way: reading backwards from a result, every decision looks inevitable. The alternatives that were live at the time are invisible, the reversals leave no residue in the final state, and the reconstruction comes out **smoother than the work ever was**. A reader of that document learns a sequence of correct steps and learns nothing about how correctness was found — which is the only part worth having.

Inverse derivation is the disciplined version. It insists on three things the intuitive method omits. It works at a grain where the before-state is recoverable, so the reconstruction has a real starting point rather than an imagined one. It grades every link by what actually supports it, so the boundary between recovered history and plausible narration stays visible. And it treats the crooked path as the substance rather than as noise to be tidied away.

The reconstruction has value on its own — it is how one studies unfamiliar high-quality work at the level of method rather than output. It also has a consumer that cannot exist without it: replaying an episode forward requires something to replay from.

## 2. Constraints & Assumptions

- **The record is the evidence, and its richness varies.** Some bodies of work carry discussion, review, and readable revision history; some carry a single flattened commit per release. The mechanism must produce a graded, honest result on both rather than a confident result on one and a fabrication on the other.
- **The subject is a completed episode, not a live plan.** The work being reconstructed is finished; nothing about it can be asked, only read.
- **The record was not written to be read this way.** Change descriptions are terse, discussions digress, and the relationship between a discussion and the changes it produced is often implicit. Recovery is inference over an incidental artifact, and the grading exists because of it.
- **Foreign material is untrusted.** A record fetched from outside carries text authored by strangers, including text that may be shaped to influence an agent reading it.
- **Licensing governs reuse, not reading.** Analysis of a record is not redistribution of it; the constraint that matters is on what leaves the reconstruction (IVD-11), not on what enters it.
- **Once materialized, the corpus is local.** Reconstruction and everything downstream of it run without network access; only corpus acquisition needs one.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **IVD-1 (The episode is the unit of reconstruction):** reconstruction operates on a **completed arc of work** — a recoverable before-state, an after-state, and the record of how the work travelled between them — never on a whole product. A product-scale reconstruction has no checkable before-state, no bounded cost, and no way to be replayed; it degrades into commentary. Episode boundaries are recorded explicitly (IVD-5) rather than inferred at read time, so two runs reconstruct the same arc.

- **IVD-2 (Every reconstructed link carries an evidentiary grade):** each link in the chain is labelled **attested** (stated in the record), **inferred** (not stated, but following from the record's content with a named basis), or **assumed** (supported by nothing; supplied to keep the chain connected). The grade travels with the link into every downstream consumer. Without grading, a reconstruction is indistinguishable from an invention, and its most confident-sounding parts are typically its least supported — the parts nobody wrote down because nobody thought them.

- **IVD-3 (The trajectory is reconstructed output, not context):** the reconstruction records **how the work moved** — the number of distinct attempts, the points where it reversed or abandoned an approach, the sequence in which parts were settled — as first-class content alongside what was produced. A reconstruction that yields only an ordered list of correct steps has deleted the search and kept the answer, which inverts the value: the answer is already visible in the artifact, and the search is the only thing the record uniquely holds.

- **IVD-4 (Reconstruction is derived, never authoritative, and never mutates its source):** the output is a derived artifact re-derivable from the record at any time. It does not edit, annotate, normalize, or stand in for the record, and no consumer treats it as the record. A reconstruction that becomes the working copy of the past silently freezes one reading of ambiguous evidence into fact.

- **IVD-5 (The corpus is an explicit registry with pinned, fingerprinted entries):** exemplar sources are listed in a **declared, human-edited, version-controlled registry**, each entry naming the source, a **pinned revision**, the **episode boundaries**, and a **content fingerprint** of what was pinned. The registry itself carries a version. Sources move under their maintainers — history is rewritten, discussions are edited, revisions are squashed — and an unpinned corpus silently changes what it contains, which invalidates every measurement taken against it without ever reporting an error. The fingerprint is what converts that silent drift into a detected mismatch. On mismatch the entry is **suspended, not repaired**: it stops satisfying reconstruction requests, measurements taken against the corpus version that contained it are marked unsound rather than recomputed, and restoring the entry requires an explicit re-pin — which produces a new corpus version, because the corpus's contents changed. Silently re-pinning to whatever the source now holds preserves the appearance of continuity across a discontinuity, which is the exact failure the fingerprint exists to expose.

- **IVD-6 (Discovery proposes; admission is explicit):** automated search MAY propose candidate sources and MAY compute their mechanical admission signals. It MUST NOT admit an entry into the registry. Admission is an explicit, recorded act. A corpus that grows by automated admission is a corpus whose composition nobody chose, and its measurements answer a question nobody asked.

- **IVD-7 (Selection is on the quality of the record, not the quality of the product):** admission is judged on whether the source's **traces support reconstruction** — legible revision history, substantive discussion, recoverable links between framing and change, runnable checks — not on the product's popularity, scale, or reputation. Popularity proxies rank by *product* success; a widely-admired product with a flattened, discussion-free history is worthless here, and an unremarkable one with a rich record is valuable. An implementation that ranks by proxy metrics will systematically acquire the wrong corpus while appearing to acquire the right one.

- **IVD-8 (Admission is a mechanical prefilter plus an irreducible human judgment):** the mechanically checkable criteria (history granularity, presence and substance of discussion, runnable checks, recoverable episode boundaries, licence permitting analysis) are computed and reported for every candidate. The judgment that the record reflects **genuine engineering search rather than a curated presentation of it** is made by a person on sampled episodes. The prefilter exists to make the human judgment affordable, never to replace it.

- **IVD-9 (Foreign record content is data, never instruction):** text recovered from an external source — discussion bodies, change descriptions, review comments, documentation, code comments — is treated as untrusted data under CP-1/CP-2. It is never executed, never followed as direction, and never allowed to alter the reconstruction procedure. Reconstruction reads adversarial material by design; a mechanism that lets the material it reads redirect it has no floor.

- **IVD-10 (Thin records degrade honestly and visibly):** when the record cannot support a link, the reconstruction marks the link **assumed** or leaves it **absent**, and reports the resulting coverage. It never fills a gap with a confident narration to keep the chain looking complete. An episode whose reconstruction is mostly assumed is reported as such and is a candidate for exclusion rather than a candidate for creative completion.

- **IVD-11 (Output is abstracted, never carried over verbatim):** what leaves a reconstruction is **method** — framing, sequencing, decision structure, the shape of the search — not the source's content. Verbatim or near-verbatim carry-over of source material into downstream artifacts is refused under the corpus originality gate (ORI-1/ORI-3), which is applied to reconstruction output rather than reimplemented. Studying work to learn how it was made is legitimate; reproducing it is a different act with different constraints, and the boundary is enforced mechanically because it is easy to cross by accident.

- **IVD-12 (A materialized corpus is offline-complete):** once entries are materialized locally, reconstruction and every downstream consumer run with no network dependency. Network access is confined to acquisition and to explicit re-pinning. A measurement instrument that reaches the network mid-run is neither reproducible nor available.

- **IVD-13 (Reconstruction is bounded and reports its incompleteness):** every reconstruction runs under a declared budget, and a run that exhausts its budget reports **which links it did not reach** rather than emitting a chain whose gaps are indistinguishable from findings. Partial coverage stated plainly is usable; partial coverage presented as complete corrupts everything computed from it.

## 4. Detailed Design

### 4.1 The chain and its direction

```
finished artifacts
   ◀── which units of work produced them
        ◀── how those units were planned and ordered
             ◀── what framing the plan answered
                  ◀── what originating intent the framing served
```

Each arrow is a recovery step with its own evidence basis, and each is graded independently (IVD-2). Grades do not propagate by inheritance: an attested unit of work may sit under an assumed framing, and that combination — strong evidence for *what* was done, none for *why* — is both common and important to represent faithfully, because it is exactly the shape that tempts narration.

### 4.2 Evidentiary grades

| Grade | Basis | Typical source | How a consumer must treat it |
| --- | --- | --- | --- |
| attested | Stated in the record | Discussion body, change description, review exchange, decision note | Usable as fact about the process |
| inferred | Follows from record content, with a named basis | Change ordering, dependency structure, revision sequence | Usable, with the basis available for challenge |
| assumed | Nothing supports it | Supplied to keep the chain connected | Never usable as evidence; visible as a gap |

The grade is not a confidence score. It is a statement about **what kind of support exists**, which is checkable, whereas a confidence number is a summary of a judgment nobody can re-derive.

### 4.3 The episode

An episode is bounded so that it yields four things without further construction:

| Yield | What it is | Why it matters |
| --- | --- | --- |
| Before-state | The subject at the revision preceding the arc | The honest starting context; without it there is no reconstruction, only description |
| After-state | The subject at the revision concluding the arc | The reference the arc is measured against |
| Executable checks | Verification introduced or exercised within the arc | An objective oracle that costs nothing to obtain |
| Trajectory | Attempts, reversals, abandonments, ordering | The recovered method (IVD-3) |

A single substantial source yields many episodes. This is what makes the grain affordable: reconstruction cost scales with the arc, not with the product.

### 4.4 The registry entry

```
entry
  ├─ source identity
  ├─ pinned revision
  ├─ episode boundaries (start revision, end revision, associated discussion)
  ├─ content fingerprint of the pinned material
  ├─ mechanical admission signals, as measured at admission
  ├─ admission record (who admitted, when, on what sampled evidence)
  └─ corpus version in which the entry became effective
```

The fingerprint answers a question the pinned revision alone cannot: a source's maintainer can rewrite the history a revision identifier points at. Without the fingerprint the corpus changes underneath every measurement ever taken against it, and nothing reports an error. With it, the change surfaces as a mismatch on the next materialization.

### 4.5 The admission funnel

```
candidate (proposed by search, by a person, or by any other means)
   ──▶ mechanical prefilter ── fails ──▶ rejected, reason recorded
   ──▶ sampled human review ── fails ──▶ rejected, reason recorded
   ──▶ explicit admission ──▶ registry entry ──▶ corpus version increments
```

Discovery sits entirely to the left of the funnel (IVD-6). The distinction is not bureaucratic: it is what keeps the corpus's composition a decision rather than a side effect, and it is what lets a measurement name the corpus version it was taken against.

### 4.6 Honest degradation on thin records

| Record condition | Response |
| --- | --- |
| Rich discussion, granular history | Full chain, mostly attested |
| Granular history, no discussion | Chain recovered to the plan level; framing and intent marked assumed |
| Flattened history, some discussion | Episode boundaries usually unrecoverable → the source fails admission (IVD-7) rather than being reconstructed badly |
| Rich record, restrictive licence | Fails admission on the licence signal; not a reconstruction problem |

The rule underneath the table is IVD-10: the mechanism reports what it could not recover. A reconstruction is allowed to be incomplete and is not allowed to disguise incompleteness.

### 4.7 Demarcation — four neighbours that are not this

- **Code comprehension** builds structural understanding of a body of code — what exists, how it connects, what a change would touch. It is a **supplier** here and answers a different question: comprehension explains the artifact, reconstruction explains the process. A perfect structural map of a codebase contains no information about what was tried and abandoned.
- **The reproduction recipe** is recorded at production time by the producer and travels with the artifact (RR-3, RR-8). It is authoritative because its author was present. A reconstruction is asserted afterwards by a reader and is graded precisely because its author was not.
- **Specialty exemplars** are authored to be references (SE-2/SE-5): fixed, convention-perfect, purpose-built. An episode is recovered from work done for its own reasons, which is why it carries reversals an authored exemplar never would — and why it can teach method that an authored artifact cannot.
- **Capability leveling** consumes reconstructions and knows nothing about how they were obtained; this spec produces them and knows nothing about models or replay. The seam is one artifact — the graded chain — crossing in one direction (IVD-4).

## 5. Implementation Notes

- Materialize once, reconstruct many times. Acquisition is the only network-bound step and the only expensive one; per-episode reconstruction against a materialized source should be cheap enough to re-run whenever the procedure changes.
- Record the mechanical admission signals **as measured at admission**, not as recomputed later. When a source's record improves or degrades over time, the difference between the admitted measurement and a current one is itself the corpus-health signal.
- Report grade distribution per reconstruction (how much attested, inferred, assumed). A corpus whose reconstructions are drifting toward *assumed* is losing its evidentiary basis, and the drift is invisible unless the distribution is a standing output rather than a per-run detail.
- Keep the trajectory representation as coarse as it can be while remaining comparable — count of attempts, location of reversals, ordering of settled parts. Fine-grained trajectory encodings tempt implementations into precision the record does not support.
- Prefer excluding a thin source over reconstructing it optimistically. Corpus composition is cheap to change early and expensive to change after measurements have accumulated against it.

## 6. Drawbacks & Alternatives

- **Hindsight is not fully eliminable.** Reading backwards from a known result biases even a graded reconstruction toward coherence. Grading bounds the damage by making the unsupported parts visible; it does not remove the bias. The structural remedy lives downstream, where a reconstruction that produces an impossibly smooth replay is treated as evidence of contamination.
- **Trajectory recovery is the weakest link.** Abandoned work is the part of a record most likely to be squashed, rebased away, or never committed. Reconstruction sees the reversals that survived the record's own tidying, which under-counts them systematically. <!-- TBD: whether an under-count correction is estimable from record-granularity signals, or whether the under-count is simply reported as a known floor -->
- **Corpus acquisition is real, recurring work.** Sources must be found, pinned, sampled, and re-pinned. This is the cost of having an instrument at all; the alternative is a corpus that costs nothing and measures nothing.
- **Alternative — reconstruct whole products:** rejected (IVD-1). No checkable before-state, unbounded cost, and no possibility of replay; the output is an essay about a codebase.
- **Alternative — accept the snapshot without history:** rejected. It removes the before-state, the trajectory, and the oracle simultaneously, leaving only the finished artifact — which is the input to comprehension, not to process recovery.
- **Alternative — automated corpus admission by proxy ranking:** rejected (IVD-6, IVD-7). Proxy metrics rank product success, which is close to uncorrelated with record quality, so the corpus fills with exactly the sources that cannot be reconstructed.
- **Alternative — treat all recovered links as equally certain:** rejected (IVD-2). It is cheaper and produces a document that reads better; it also destroys the only thing separating recovered history from fiction.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[COMPREHENSION]` | `.design/main/specifications/l1-code-intelligence.md` | The structural-understanding supplier and the demarcation boundary (§4.7) |
| `[CONSUMER]` | `.design/main/specifications/l1-capability-leveling.md` | The downstream loop; defines what the graded chain must carry |
| `[RECIPE]` | `.design/main/specifications/l1-reproduction-recipe.md` | The forward twin; RR-3 content-addressed identity reused by IVD-5 |
| `[PROVENANCE]` | `.design/main/specifications/l1-context-provenance.md` | CP-1/CP-2 untrusted-by-default, the basis of IVD-9 |
| `[ORIGINALITY]` | `.design/main/specifications/l1-corpus-originality.md` | ORI-1/ORI-3 gate applied to reconstruction output (IVD-11) |
| `[EXEMPLARS]` | `.design/main/specifications/l1-specialty-exemplars.md` | The authored-fixture sibling; SE-2/SE-5 grading discipline |
| `[DECLARATION]` | `.design/main/specifications/l1-declarative-configuration.md` | DC-1 single declaration, the registry's form (IVD-5) |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-20 | Core Team | Initial concept: reconstructing a production chain backwards from finished work plus its record. Episode as the unit, with a recoverable before-state, after-state, executable checks, and trajectory (IVD-1); every link graded attested/inferred/assumed, with grades non-inheriting along the chain (IVD-2); the trajectory — attempts, reversals, abandonments — as first-class output rather than context, on the grounds that the artifact already shows the answer and only the record shows the search (IVD-3); reconstruction derived, re-derivable, and never authoritative over or mutating its source (IVD-4); an explicit human-edited registry with pinned revisions, episode boundaries, content fingerprints, and its own version, closing the silent-corpus-drift hole, with a fingerprint mismatch **suspending** the entry and marking dependent measurements unsound rather than silently re-pinning to whatever the source now holds (IVD-5); automated discovery permitted as a candidate proposer and forbidden as an admitter (IVD-6); selection on record quality rather than product popularity, with the proxy-metric failure mode stated (IVD-7); mechanical prefilter plus an irreducible human judgment on sampled episodes (IVD-8); foreign record content treated as untrusted data under CP-1/CP-2 (IVD-9); honest degradation and reported coverage on thin records, never narrative gap-filling (IVD-10); output abstracted to method with verbatim carry-over refused via the ORI-1/ORI-3 gate (IVD-11); materialized corpora offline-complete (IVD-12); bounded runs that report unreached links rather than disguising gaps (IVD-13). Demarcated in §4.7 from code comprehension (artifact vs process), the reproduction recipe (recorded by the producer vs asserted by a reader), authored specialty exemplars (purpose-built vs recovered), and capability leveling (the one-directional consumer seam). |
