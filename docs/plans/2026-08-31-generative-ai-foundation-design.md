# Generative AI foundation design

## Outcome

New product bets can add one useful generative behaviour without replacing the
deterministic product core, crossing a declared data boundary, or coupling the
starter to one model vendor. The foundation stays inert until a product wires a
runtime.

## Decisions

1. Add a WASM-safe `rv2-ai` crate containing provider-neutral request, draft,
   citation, review, and validation contracts.
2. Default every request to `LocalOnly`. Remote inference is an explicit product
   decision and must carry a consent path; the starter ships no remote adapter,
   API key, model download, or background inference.
3. Require grounded facts with stable identifiers. A generated draft may cite
   only facts supplied in its request, and invalid or oversized output fails
   closed before display.
4. Treat every output as a draft. Product code must expose accept, edit, and
   reject actions before persistence, export, payment, submission, or another
   irreversible action.
5. Keep deterministic formulas, eligibility rules, prices, legal confirmations,
   consent, and final decisions outside model authority.
6. Keep runtime adapters in product-owned crates. Browser workers, native model
   runtimes, and remote services have different async, binary-size, and privacy
   costs; forcing one into the starter would make every bet pay for it.

## Data flow

Product-owned inputs and first-party facts become a validated
`GenerationRequest`. A runtime implementing `DraftGenerator` produces a
`GenerationCandidate`. `validate_candidate` checks output length, non-empty
content, and citation integrity, returning a `ReviewableDraft`. Product UI then
records explicit accept, edit, or reject behaviour. Deterministic fallback stays
available whenever runtime is absent or validation fails.

No generated text enters an AllSource event automatically. Each product defines
its own accepted-draft event only when persistence supports its essential value
circuit.

## Weekly bet use

Weekly portfolio work selects one active bet and one smallest user-visible
generative enhancement tied to that bet's fixed gate. Preferred behaviours draft,
explain, classify, or transform customer-owned material. They do not invent
facts, provide regulated conclusions, or replace paid deterministic output.

Qualified AI evidence is downstream behaviour such as an accepted, materially
edited, or exported draft. Generation count, clicks, praise, and model quality
claims remain diagnostic only and never promote a bet.

## Error handling

- Missing instruction, facts, or output budget: reject request locally.
- Empty, oversized, or ungrounded candidate: reject before display.
- Runtime unavailable: show deterministic/manual path; never hide core value.
- Remote boundary requested without product-specific consent: product must not
  construct or dispatch request.
- Unsupported high-stakes use: keep model outside decision path.

## Verification

- Unit tests cover request construction, local-only default, output bounds,
  known citations, and explicit review decisions.
- `cargo xtask ci` cross-compiles `rv2-ai` to `wasm32-unknown-unknown` and proves
  server-only crates remain unreachable from both Dioxus apps.
- Weekly template and starter validator require AI foundation for placements
  made under the new foundation contract while preserving older repositories.
