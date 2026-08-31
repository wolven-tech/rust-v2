# Generative AI contract

**Status:** Accepted foundation decision.

This extends, without replacing, the boundaries in
[`001-rust-v2-allsource-foundation.md`](001-rust-v2-allsource-foundation.md).
Product intent and evidence rules live in
[`../plans/2026-08-31-generative-ai-foundation-design.md`](../plans/2026-08-31-generative-ai-foundation-design.md).

## Decisions

| # | Decision | Why |
|---|---|---|
| AI1 | `rv2-ai` is WASM-safe, provider-neutral, and contains contracts only. | Browser, mobile, and server products share safety rules without sharing one runtime or binary cost. |
| AI2 | Requests default to `LocalOnly`; remote use requires an explicit constructor and rejects local-only facts. | Data must not cross a boundary because a provider happened to be configured. |
| AI3 | Candidates must cite request-owned fact ids and pass length and integrity checks before display. | Model-authored URLs or free text cannot become evidence. |
| AI4 | Every valid candidate remains a reviewable draft. | A model cannot accept terms, submit, pay, persist, or make another irreversible choice for a person. |
| AI5 | Runtime adapters stay product-owned and deterministic/manual value remains usable. | A starter dependency must not make every bet depend on inference availability, network, or one vendor. |

## Crate boundary

`rv2-ai` is layer 1. Allowed dependencies are allocation-safe Rust, `serde`,
and error types. It must cross-compile to `wasm32-unknown-unknown` under
`cargo xtask ci`.

Runtime placement depends on product need:

- browser worker or WASM model adapter: product-owned WASM-safe crate;
- native mobile model adapter: product-owned native crate behind target cfg;
- remote provider adapter: server-only product crate, explicit consent path,
  rustls transport, and no provider secret in either Dioxus bundle;
- deterministic fallback: domain or product crate, always available when it is
  part of paid core value.

No runtime adapter belongs in `rv2-domain`: domain formulas and eligibility
rules stay deterministic. No provider response becomes an AllSource event by
default. Products may record explicit human accept, edit, or reject outcomes
using their own additive event schema when that evidence serves the fixed bet
gate.

## Dispatch sequence

1. Product turns customer-owned inputs and sourceable facts into
   `GroundedFact` values.
2. Product constructs local request, or remote request after explicit consent.
3. Product runtime implements `DraftGenerator` and returns untrusted
   `GenerationCandidate`.
4. Product calls `validate_candidate` against original request.
5. UI shows `ReviewableDraft` with sources and accept, edit, reject controls.
6. Only explicit human action may cross persistence or external-action boundary.

Validation failure, unavailable inference, or missing consent returns customer
to deterministic/manual path. It must not silently retry against a remote model.
