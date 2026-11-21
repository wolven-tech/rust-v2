# rv2-affect: affectUI's free components, ported

`crates/rv2-affect` is a Dioxus port of the 74 MIT-licensed items of
[affectUI](https://www.affectui.com): 51 components, 4 motion effects, 2 page
blocks and 17 illustrations. The licence notice is
`crates/rv2-affect/LICENSE-affectui`.

## Decisions

| # | Decision | Why |
|---|---|---|
| AF1 | A new crate, not a module of `rv2-ui`. | `rv2-ui` is a closed instruction set grown one consumer at a time (UI8) and styled with Tailwind against one register source (UI2). affectUI is a catalogue whose every item carries its own stylesheet and its own token set. Merging them would either break UI8 seventy times or force every item through a Tailwind rewrite that loses the original's look. |
| AF2 | `rv2-affect` is outside the instruction set, as `rv2_ui::motion` is (UI9). | It has no in-repo consumer yet. An app may use it, but the ISA's rules (UI5–UI7) are stated against `rv2-ui` and are not extended to this crate. |
| AF3 | CSS is the original's, moved verbatim from each TSX template string into `assets/<slug>.css`. Class names (`ui-*`), data attributes and `--aui-*` variables are unchanged. | affectUI's theming docs then apply as written, and a diff against an upstream release stays readable. |
| AF4 | Each `--aui-*` hook is mapped onto an rv2 register by `assets/rv2-bridge.css`, linked by the `Rv2Theme` component. Without it the originals' hex fallbacks apply. | UI1/UI2 ask that colours come from one register source. The bridge is the values-only seam UI3 describes: an app theming rv2's registers themes these components too. |
| AF5 | No `web-sys`, no JS package. Measurement and focus use `onmounted`; Web Animations, clipboard, `matchMedia`, timers and document-level listeners use `document::eval`. | Same dependency line as `rv2-ui`. Where a port loses fidelity the module doc says what, under "Port losses". |
| AF6 | WASM-safe, and added to the `wasm32 boundary` check in `cargo xtask ci`. | Both apps compile to `wasm32-unknown-unknown`; a crate they may take must too. |
| AF7 | affectUI's 160 Pro items are rebuilt clean-room in a separate crate, `rv2-kit`, never ported. | Their source is licensed, and this repository is public under MIT. Function is not protected; code, CSS, art and copy are. A spec room reads only the public pages and writes functional specs (`docs/specs/rv2-kit/`) with no styling values, prose or art; a clean room implements from those specs alone, in rv2's own look. The specs are the provenance record. |

## Not ported

- **affectUI Pro** (160 items). Rebuilt clean-room instead; see AF7.
- **The `-clean` variants.** Each free item ships a second, plainer variant;
  only the default one is ported.

## Previewing

`dx serve --package app --example affect_gallery --platform web` renders every
component on one page, with a light/dark switch. The gate renders no page
(see `AGENTS.md`), so open it after changing a component.
