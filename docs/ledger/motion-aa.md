# Motion kit — WCAG 2.1 AA

An autoresearch loop over `rv2_ui::motion` and the `/motion` showcase.

## Setup

**Frozen corpus.** The `/motion` page served by `apps/web` at `15a7a6d`, at
1280×900, plus its `prefers-reduced-motion: reduce` state. Page *content* was not
changed during the loop — only how it is presented.

**Scalar (lower is better).** AA questions that are failing **or unanswered**:

```
violation nodes + unresolved color-contrast nodes
```

as reported by axe-core 4.10.2 under `wcag2a`, `wcag2aa`, `wcag21a`, `wcag21aa`.

Counting `incomplete` alongside `violations` is the decision that made this loop
worth running. The first scalar was violations alone, and it read **1** — which
looks like "almost compliant" and is really "axe declined to judge the four
places that actually mattered". A checker that cannot compute a contrast ratio
has not passed it. `incomplete` is an open question, and open questions are what
this loop is for.

Deterministic: axe reads computed CSS, not rendered pixels, so animation does not
perturb it. No noise band needed.

**Gate.** All of:

1. `cargo xtask ci` green.
2. All seven components present and interactive on the page.
3. No component deleted. Deleting the blob would score beautifully.
4. Every infinite animation covered by the `prefers-reduced-motion` rule
   (WCAG 2.2.2 mitigation; axe cannot test it).
5. **No unsubstituted template placeholder in the rendered text** — added
   mid-loop, see iteration 1b.
6. Looks right in a screenshot.

**Baseline: 5.** One `html-has-lang` violation, four unresolved contrast nodes
(three on the hologram, one on the fur patch).

## Ledger

| # | Proposal | Scalar | Δ vs baseline | Verdict |
|---|---|---|---|---|
| 0 | Baseline | 5 | — | — |
| 1 | Custom `index.html` with `lang="en"` — dx emits `<html>` with no language, failing 3.1.1 | 4 | −1 | **Keep** — no Dioxus.toml setting and no Rust API exists, so the template is the supported route |
| 1b | *(regression found by screenshot)* `{script_include}` is not a dx 0.7 placeholder; it rendered as literal text at the bottom of every page. Removed; gate strengthened to reject leaked placeholders | 4 | −1 | **Keep the fix** — iteration 1 as first applied was a defect the gate did not catch |
| 2 | Opaque `bg-slate-900` plate under the hologram's content | 1 | −4 | **Keep** — three nodes resolved; a real foil card works this way, art shimmers and the text box does not |
| 3 | Opaque chip behind the fur caption | **0** | **−5 (−100%)** | **Keep** — the strand highlights swing several stops of lightness under that text as it ruffles |

One measurement was taken and thrown away: iteration 2's first run reported no
change, and the DOM showed the plate still carrying its old classes — dx had not
rebuilt the Rust. An invalid measurement is not a result, so no row was written
for it. Hot reload was unreliable for structural RSX changes throughout; every
row above was measured after a clean `dx serve` restart.

## Net

**5 → 0.** All 52 contrast nodes on the page are now *proven* to pass rather than
unverifiable. Zero AA violations, zero unresolved contrast, `lang="en"`, all
seven components intact.

## What this does NOT claim

**Scalar 0 is not "AA compliant".** axe decides roughly 40% of WCAG
automatically. What it covered here is now clean; the rest has not been audited:
keyboard traversal of the whole page, focus order, reflow at 320 px (1.4.10),
non-text contrast of the cord and knob (1.4.11), and whether a screen reader is
told anything useful when the blob's mood changes.

## Open

- **WCAG 2.2.2 (Pause, Stop, Hide).** Two animations loop forever: the blob's
  idle wobble and the hologram's sheen. `prefers-reduced-motion` stops both, and
  the gate proves it — but that is an OS preference, not the on-page mechanism
  2.2.2 asks for. Two honest fixes, and choosing between them is a design call
  rather than a loop decision:
  1. Make both finite and under five seconds, which satisfies the criterion's own
     exception outright. Costs the "the light keeps sweeping on its own"
     behaviour, which was a deliberate design choice.
  2. Add a page-level motion toggle. Keeps the behaviour, adds a control and the
     state to drive it.
- **`apps/app` has the same `lang` defect** and is outside the frozen corpus. It
  needs the same `index.html`. Not done here, because changing the corpus
  mid-loop invalidates the numbers.
- **The `{style_include}` / `{app_title}` placeholders are now pinned.** This
  workspace depends on two dx template internals that are not, as far as the
  docs go, a stable contract. A dx upgrade should re-check the rendered shell.
