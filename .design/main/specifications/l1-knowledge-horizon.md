# Knowledge Horizon

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

A model's parametric knowledge ends at a fixed horizon while the session runs at *now* — and the most damaging failure in that gap is a confident assertion about something the model half-recognizes. This micro-spec fixes the generation-side discipline: the horizon is declared, *now* is injected, unrecognized entities force retrieval before assertion, and queries are anchored to the present. It is the generation-side complement of claim verification (which checks text already produced); this contract prevents the confabulated claim from being produced at all.

## Motivation

Verification-side auditing (claim verification) catches a confabulated claim only after it exists — and only when sources were gathered. The cheapest, highest-leverage point to stop confabulation is before assertion: recognize the horizon-gap condition and route through retrieval. Without this contract, each surface improvises its own freshness heuristics and the failure repeats: a confidently stale answer about something the model half-recognizes.

## Related Specifications

- [l1-claim-verification.md](l1-claim-verification.md) - Verification-side sibling: CV audits produced claims against sources; KH gates assertion before production.
- [l1-retrieval-evaluation.md](l1-retrieval-evaluation.md) - Measures the retrieval quality KH-2 depends on.
- [l1-cache-stable-context.md](l1-cache-stable-context.md) - The injected current date is volatile content and lives outside the stable prefix.

## Core Invariants

- **KH-1 (Declared horizon, injected now):** every model-facing session declares the model's knowledge horizon and injects the host's current date as runtime context (volatile, never baked into reusable prefix content). Reasoning about "recent" is always relative to the injected *now*, not to the horizon.
- **KH-2 (Retrieval before assertion):** a claim about an entity, release, event, or current status that the generator does not recognize — or that plausibly postdates the horizon — MUST trigger source retrieval before the assertion is made; if retrieval is unavailable, the claim is explicitly marked unverified rather than asserted. Recognizing a franchise, author, or series is not knowing its latest member. Retrieval costs little; a confabulated answer costs the user's trust.
- **KH-3 (Now-anchored queries):** retrieval queries are formulated against the injected current date, never the horizon date — a query stamped with the horizon's year silently returns stale results. Present-tense questions that sound settled ("is X still Y", "who currently holds Z") are treated as current-status queries and routed through retrieval.
- **KH-4 (Post-retrieval trust calibration):** retrieved evidence outranks parametric memory for post-horizon facts, including surprising results — with declared skepticism classes (manipulation-prone, consensus-lacking, heavily optimized topics) where corroboration is required before adoption.

## Drawbacks

- Retrieval gating adds latency to a class of answers; accepted — the gated class is exactly where parametric answers are least trustworthy. Hosts without any retrieval capability degrade honestly via the KH-2 unverified marking.

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-07-13 | Core Team | Initial micro-spec — KH-1…KH-4: declared horizon + injected now, retrieval-before-assertion for unrecognized/post-horizon entities, now-anchored query formulation, post-retrieval trust calibration with skepticism classes. |
