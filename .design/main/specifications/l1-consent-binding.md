# Consent Binding

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

Action gating decides **how much** authorization friction an act must pass through. This concept decides the question that comes immediately after and is currently unasked: **what does a granted consent actually cover, and when does it lapse?**

The failure it closes is quiet and structural. Consent is naturally given to an *intention* — "yes, run the test suite", "yes, this deployment script is fine" — while what executes is a **resolved invocation**: a specific operation, with specific arguments, in a specific working location, through a specific interpreter, under a specific environment that decides what its names resolve to. Any of those can change without the part the approver read changing at all. The same authored line, re-run from a different location or under a different environment, is a different act; if the grant was recorded against the intention, that different act inherits an approval nobody gave it.

The contract has three moves. A grant **binds the fully resolved form** of the act, not its name or description. The approver is shown **that resolved form**, because consent obtained for the authored text is consent for something the system will not do. And any change to a bound element **lapses the grant** — there is no partial reuse and no close-enough match. Around those sits one piece of honesty that must be stated rather than implied: a grant authorizes an act; it constrains nothing about what that act then does, and it does not reach the act's own dependencies.

## Related Specifications

- [l1-action-gating.md](l1-action-gating.md) — the sibling half. AG decides the **tier** (auto / confirm / approval) from consequence; this decides the **identity and lifetime** of whatever grant that tier produces. AG-6's scoped, revocable, human-ratified de-escalation rule is exactly the grant CB-1 gives an identity to, and CB-3 is what stops it widening by drift.
- [l1-security.md](l1-security.md) — SEC-9/SEC-10: authority is human-rooted and never self-granted; consent records are authority records and inherit that plane's rules.
- [l1-interception-model.md](l1-interception-model.md) — INT-2 check-before-use ordering and INT-3 fail-closed direction; a grant lookup is a *decide*-class evaluation and fails closed on ambiguity.
- [l1-execution-sandbox.md](l1-execution-sandbox.md) — the confinement CB-5 explicitly declines to be. Consent is authorization; isolation is a separate mechanism, and conflating them misleads the person being asked to decide.
- [l1-provenance-taint.md](l1-provenance-taint.md) — the reviewed material is untrusted data (CB-8); its text can never instruct the reviewing actor, least of all into approving itself.
- [l1-tool-receipts.md](l1-tool-receipts.md) — receipts prove an act *happened*; a grant states an act was *permitted*. Neither implies the other, and neither proves the act was correct.
- [l1-acceptance-oracle.md](l1-acceptance-oracle.md) — CB-9's counterpart: consent says an act may run, an oracle says whether its outcome is right. Approval is never any part of the proof.
- [l1-reproduction-recipe.md](l1-reproduction-recipe.md) — the recipe's ambient-configuration layer is the same insight applied to artifacts: what was in force silently changed the outcome, so it belongs in the record.
- [l1-environment-lifecycle.md](l1-environment-lifecycle.md) — the ambient environment that makes an authored line resolve differently on different hosts; a bound element of every grant.
- [l1-declarative-configuration.md](l1-declarative-configuration.md) — DC-10: configuration never widens authority; a value entering through a configuration surface cannot enlarge a grant.
- [l1-resource-sharing.md](l1-resource-sharing.md) — RS-9 non-escalating grants; the sharing plane's version of the same rule at the resource grain.

## 1. Motivation

The project has a well-formed gating ladder and a well-formed audit trail, and both are indifferent to the question of *identity*. AG-7 records that an action was approved and by whom. Nothing states what the approval was **for** with enough precision to decide whether the *next* attempt is the same act.

That gap opens in ordinary use, without anyone acting badly:

- **Re-resolution.** An approved operation is invoked again from a different working location, or through a different interpreter, or on a host where a bare name resolves to a different program. The text is byte-identical; the act is not.
- **Environment drift.** The set of locations searched for a name changes between one session and the next. The approver approved what the name meant then.
- **Argument creep.** A grant recorded against an operation's *name* silently covers every future set of arguments handed to that name, including the destructive ones.
- **The authored/resolved gap.** The approver is shown the line as written, with placeholders unexpanded and defaults unstated, and consents to a form that is not what runs.
- **Transitive change.** A grant covers a byte-identical invocation whose called script, fixture, or dependency has since changed. This one cannot be closed by binding harder — and pretending otherwise is worse than disclosing it.

Left unnamed, each of these produces the same outcome: an act executes under an approval that its approver did not give, and the audit trail records a legitimate-looking authorization. AG-5's friction-fatigue rule makes this materially worse rather than better — a system that correctly minimizes prompts is a system where an inherited grant is *more* likely to be the thing that lets an act through unread.

## 2. Constraints & Assumptions

- **Consent is scarce.** Every re-consent spends the approver's attention, which AG-5 correctly treats as the resource to protect. Binding must be precise enough to be meaningful and stable enough not to re-prompt on noise.
- **Resolution is knowable before execution.** The system can determine the resolved form of an act before performing it; otherwise there is nothing to show and nothing to bind.
- **Transitive identity is not free.** Hashing everything an act might read is impractical in general and unbounded in the limit. The boundary is drawn where it can be honestly held.
- **A grant is not a capability.** It permits one identified act; it does not confer standing authority, does not survive revocation, and cannot be widened by its holder.
- **Records are local and principal-owned.** Grant records live under the granting principal's control, consistent with the project's local-first, human-rooted authority plane.

## 3. Core Invariants

Rules every Layer 2 implementation MUST NOT violate. They are technology-neutral.

- **CB-1 (Consent binds a resolved invocation, never an intention):** a grant attaches to the **fully resolved form** of the act — the operation and its actual arguments, the working location, the interpreter or runtime that will execute it, the ambient environment that decides how its names resolve, and the limits it will run under. It does **not** attach to a name, a category, a tool, a description, or the author's summary of intent. "Approved: run tests" is not a grant; it is a sentence.

- **CB-2 (What is shown is what will run):** the approver is presented the **resolved** form, not the authored one. Where the two differ — an argument expanded, a location defaulted, an interpreter chosen by the platform, a bare name resolved through the ambient environment — presenting the authored form obtains consent for an act the system will not perform. The difference is exactly where the dangerous cases live, so it is the part that must be visible.

- **CB-3 (Any change to a bound element lapses the grant):** if any element of the bound identity differs at the next attempt, the earlier grant **does not apply** and consent is requested again. There is no partial match, no nearest-neighbour reuse, and no operator judgement that a difference is immaterial. A grant that stretches to cover an act it was not given for is indistinguishable, at the moment it matters, from no gate at all.

- **CB-4 (First encounter discloses, it does not perform):** an act whose resolved identity matches no grant is **disclosed and stopped** — its resolved form is surfaced, and the act does not execute. This preview is a property of the *unmatched* state, not a mode of the system: once a matching grant exists, the same path executes. A design that lets an actor treat the ordinary path as a permanent dry run has produced a gate that surprises exactly once.

- **CB-5 (Consent is authorization, never containment):** a grant permits an act; it constrains **nothing** about what the act then does. The permitted act runs with whatever ambient reach its executing context has — files, network, credentials, other processes — and may exceed, in effect, anything its resolved form suggests. Where reach must be limited, that is a confinement mechanism's job. A surface that presents consent as though it were isolation misleads precisely the person it is asking to decide.

- **CB-6 (The coverage boundary is stated, not implied — consent does not reach transitive inputs):** a grant binds the invocation, **not what the invocation reads**. A called script, a loaded fixture, a resolved dependency, a generated file may all change beneath a byte-identical invocation and will execute under the existing grant. This gap MUST be **disclosed** rather than papered over. Where a workflow needs those inputs' identity enforced, that identity is made **part of the bound invocation itself** and validated by something the grant already covers — a deliberate, author-designed extension of the boundary, never an implied property of it.

- **CB-7 (Grant records are principal-owned, integrity-checked, and fail closed):** grant records live **outside the material they authorize**, under the granting principal's own control, and are honoured only when their integrity is intact. A store the reviewed material can itself write is not consent — the material would be authorizing itself. Anything ambiguous — a record that cannot be validated, a store whose ownership cannot be established, a substituted or duplicated record — **fails closed**, denying the act rather than assuming the grant.

- **CB-8 (Material under review is data, never instruction):** the content being consented to — an inherited definition, a title, a description, an output the act produced — is **read as data**. It never issues instructions to the reviewing actor, and specifically can never cause its own approval, widen its own grant, request that the gate be disabled, or install the mechanism that would approve it. An act's own output is the least trustworthy possible argument for granting it.

- **CB-9 (Consent is not evidence of correctness):** a grant states that an act was **permitted to run**. It says nothing about whether the act does what its description claims, whether its outcome is right, or whether the work is complete. "The user approved it" MUST NOT appear as any part of the proof of an outcome — that is the acceptance oracle's question, and borrowing the approver's authority to answer it is a category error that reads as diligence.

- **CB-10 (Grants and lapses are both auditable):** every grant records the **resolved identity** it covers, the principal who gave it, when, and through which surface; every lapse records **which element changed**. Answering "what exactly was approved, by whom, and why did it stop applying" is a first-class capability, not a reconstruction exercise — a grant whose scope cannot be stated after the fact was never scoped.

- **CB-11 (A grant is narrowable, never self-widening):** a grant may be revoked, narrowed, or given a shorter life by its granting principal at any time. It can **never** be widened, extended, renewed, or re-scoped by the actor that holds it, by the act it authorizes, or by any automated process acting on their behalf — including by re-recording the same act under a broader identity. Widening is a new consent decision, made by the principal, through the gating ladder.

> L2 specs cannot reach RFC status until all invariants here are addressed in their "Invariant Compliance" section.

## 4. Detailed Design

### 4.1 The bound identity

```text
[REFERENCE]
identity(act) := {
    operation,            // the thing to be done, exactly as it will be invoked
    arguments,            // fully expanded — no placeholders, no unstated defaults
    location,             // the resolved working location
    interpreter,          // the runtime/shell that will execute it, as resolved
    environment,          // the ambient resolution context: what names mean here
    limits,               // time, size, and resource bounds it will run under
    platform,             // the host class it resolves against
}

attempt(act):
    id := identity(resolve(act))          // CB-1 — resolution happens before the lookup
    grant := lookup(id)                   // exact match only — CB-3
    if grant is None:
        disclose(id)                      // CB-2 / CB-4 — show the RESOLVED form
        return NOT_PERFORMED              // consent is requested; the act does not run
    if not integrity_ok(grant):
        return DENY                       // CB-7 — fail closed, never assume
    return PERFORM(act)                   // CB-5 — permitted; not contained, not proven
```

The list is not arbitrary: each element is something that can change the *meaning* of an otherwise identical act while leaving the text the approver read untouched. An element that cannot do that does not belong in the identity, because binding it would spend consent on noise and drive the approver toward the blanket approval AG-5 forbids.

### 4.2 The authored/resolved gap (CB-2)

| Approver is shown | System actually performs | Consequence of showing the authored form |
| --- | --- | --- |
| the operation with placeholders | the operation with placeholders expanded | The approver evaluated a template, not an act |
| the operation with no location | the operation in a location chosen by a default | The blast radius was never visible |
| a bare program name | whatever that name resolves to on this host, now | The approver approved a different program |
| the operation without its limits | the operation under limits that change its behaviour | A bounded act and an unbounded one look identical |

Every row is a case where the approver's answer was reasonable and the question was wrong. CB-2 is the invariant that makes the question match the act.

### 4.3 The transitive boundary (CB-6), and why it is disclosed rather than closed

An honest system states where its guarantee ends. Binding the invocation is achievable and stable; binding everything the invocation transitively reads is neither — the closure is unbounded, changes constantly, and would re-prompt on every unrelated dependency update until the approver disabled the gate entirely.

So the boundary is drawn at the invocation and **said out loud**. The consequence is concrete and must be understood by anyone relying on a grant: *a byte-identical act whose called script changed yesterday runs today under yesterday's approval.* Two things follow. Changed dependencies are re-inspected as a matter of practice, not assumed covered. And where a workflow genuinely requires dependency identity to be enforced by mechanism, the author **puts that identity inside the bound invocation** — validated by something already inside the grant — which is a deliberate extension the approver can see, rather than a property the system silently fails to have.

### 4.4 Demarcation

| Layer | Answers | This spec does not |
| --- | --- | --- |
| l1-action-gating | *How much* friction does this act deserve? | assign tiers or classify consequence |
| **this spec** | *What* was consented to, and *when* does it lapse? | — |
| l1-execution-sandbox | What may the act reach once it runs? | constrain reach (CB-5) |
| l1-tool-receipts | Did the act actually happen, with this result? | prove occurrence |
| l1-acceptance-oracle | Is the outcome right? | prove correctness (CB-9) |

The five compose in one direction: gating decides the friction, consent binding decides the grant's identity and life, confinement bounds the act, receipts prove it occurred, and the oracle judges the result. Each is silent about the others' questions, and CB-5/CB-9 exist to keep this one from being read as answering any of them.

### 4.5 Interaction with learned de-escalation (AG-6)

AG-6 permits a proven-safe action to be de-escalated to a lower tier through a scoped, revocable, human-ratified rule. That rule is a **grant** and takes this contract's identity: it covers the resolved form it was ratified for, lapses when any bound element changes (CB-3), and can be narrowed but never widened by its holder (CB-11). This is what keeps de-escalation from becoming the drift path — the mechanism designed to spend less of the approver's attention is precisely the one that must be most exact about what it covers.

## 5. Implementation Notes

1. **Resolve-then-look-up** — resolution must precede the grant lookup, or the identity is computed from something other than what will run (CB-1). This ordering is the whole design and is easy to get backwards.
2. **Disclosure surface** — render the resolved identity element by element, marking each element that differs from the authored form; those are the ones worth the approver's attention (CB-2).
3. **Record store** — principal-owned, outside the material it authorizes, integrity-validated on read, ambiguity denied (CB-7). Never place the store where the reviewed material can write.
4. **Lapse diagnostics** — when a grant lapses, name the element that changed (CB-10). "This needs approval again" without a reason trains the approver to grant reflexively, which is the AG-5 failure arriving through the audit trail.
5. **Disclose the boundary at the point of decision** (CB-6) — the transitive gap belongs on the consent surface, not only in documentation; the approver is the person whose model of coverage must be correct.

## 6. Drawbacks & Alternatives

- **Precise binding costs prompts.** A stricter identity lapses more often. Accepted, and bounded deliberately: the identity contains only elements that can change the act's meaning (§4.1). An identity that included volatile-but-harmless detail would re-prompt on noise and drive the approver toward blanket approval — the AG-5 failure, reached by over-correcting.
- **The transitive gap is real and unclosed.** By design (CB-6). Disclosed honestly, with a named path for workflows that need more. The alternative — implying coverage that does not exist — is the more dangerous position, since it is trusted precisely where it is wrong.
- **Alternative — bind consent to a category ("shell commands", "file writes").** Rejected (CB-1): a category grant authorizes every future member of the category, including the ones nobody would have approved. This is the blanket approval AG-5 names as the friction-fatigue endpoint, arriving by design instead of by fatigue.
- **Alternative — expire grants on a timer instead of on change.** Rejected as the primary mechanism: time is uncorrelated with meaning. An act unchanged for a year is the same act; an act changed a second ago is not. A time bound may be layered on top as a policy choice, but change is the trigger.
- **Alternative — fold into action gating.** Rejected: AG's subject is *consequence* and its output is a tier; this spec's subject is *identity* and its output is a scope. Merging them would produce a layer that answers "how much friction" precisely and "for what act" vaguely — which is the present state, and is what this exists to fix.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[GATING]` | `.design/main/specifications/l1-action-gating.md` | The tier ladder; AG-6 de-escalation grants this contract scopes |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | SEC-9/SEC-10 human-rooted, never-self-granted authority |
| `[SANDBOX]` | `.design/main/specifications/l1-execution-sandbox.md` | The confinement CB-5 declines to be |
| `[TAINT]` | `.design/main/specifications/l1-provenance-taint.md` | Reviewed material as untrusted data (CB-8) |
| `[ORACLE]` | `.design/main/specifications/l1-acceptance-oracle.md` | Correctness, which consent never proves (CB-9) |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-08-25 | Core Team | Initial concept — the identity and lifetime of consent, the question action-gating leaves unasked after deciding how much friction an act deserves. Consent binds the **fully resolved invocation** (operation, expanded arguments, location, interpreter, ambient resolution environment, limits, platform), never a name, category, or intention (CB-1); the approver is shown the **resolved** form, since consent for the authored text is consent for an act the system will not perform (CB-2); any change to a bound element lapses the grant, with no partial reuse or close-enough match (CB-3); an unmatched act is disclosed and stopped, a property of the unmatched state and never a permanent dry-run mode (CB-4); consent is authorization and never containment — the permitted act keeps its full ambient reach, and presenting a grant as isolation misleads the approver (CB-5); the coverage boundary is **disclosed, not implied** — a grant does not reach called scripts, fixtures, or dependencies, so a byte-identical act runs under an old grant when its inputs change, and workflows needing that enforced put the inputs' identity inside the bound invocation (CB-6); grant records are principal-owned, stored outside the material they authorize, integrity-checked, and fail closed (CB-7); reviewed material is data and can never cause its own approval or install its own approver (CB-8); consent is never evidence of correctness (CB-9); grants and lapses are both auditable, with the lapse naming the changed element (CB-10); grants are narrowable by the principal and never self-widening by the holder or the act (CB-11). Scopes AG-6 learned de-escalation so the mechanism that spends least approver attention is the most exact about what it covers. Concept-only. |
