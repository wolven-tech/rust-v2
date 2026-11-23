---
name: rv2-kit
description: Build or change Dioxus UI on the rv2-ui component kit (rust-v2 and every product forked from it or depending on it). Use before writing any RSX in apps/*/src, when choosing a component for an intent ("a delete button", "a figure with a trend", "ask before acting"), when `apps speak the kit` / `rv2-kit check` fails, or when asked what the kit offers.
---

# Building UI on rv2-ui

An app does not style elements. It states intent through `rv2-ui`'s closed
vocabulary: **instructions** (components), **operands** (enums such as
`Variant`, `Verdict`, `Width`) and **registers** (colour tokens named for their
job). `class` carries layout only, never colour, type or radius (UI5). The
rules are UI1–UI9 in `docs/architecture/003-ui-semantic-isa.md`.

## Install once

```sh
cargo install --git https://github.com/wolven-tech/rust-v2 rv2-kit
```

In a repository that pins `rv2-ui` by git revision, install the same revision
(`--rev <sha>`) so the embedded index describes the kit you compile against.
Inside a workspace that carries `crates/rv2-ui` (rust-v2 and its forks),
`find`/`show`/`list` read that copy's source directly.

## The loop

1. **Find the instruction for the intent.** `rv2-kit find "<what the UI must
   do>"`, or `rv2-kit list` for every intent. Search by job, not by HTML tag.
2. **Read it before using it.** `rv2-kit show <Name>` prints the props (which
   are required, what the defaults are), a working example from `/kit`, the
   tests that pin its behaviour, and how to build the operands and parts it
   takes. Copy the example's shape; do not guess prop names.
3. **Compose.** Instructions inside instructions. If no operand gives the look
   you need, that is a missing operand to add to the kit, not a `class` to
   override (UI5). If no instruction fits, say so; do not hand-roll markup.
4. **Run the gate.** `rv2-kit check` from the workspace root (it scans
   `apps/*/src`; add `--src <dir>` for other layouts). Each finding names the
   instructions that replace the raw element. A raw element that is genuinely
   the subject of the page (a skip link, demonstrated markup) is marked
   `isa-raw-ok: <why>` on its line or the comment directly above.
5. **Open the page.** No test renders it. `dx serve --package <app> --platform
   web` and use the thing you built, at phone width too. A login form once
   shipped unable to submit with every test green.

## In rust-v2 itself

- `cargo xtask ci` runs `rv2-kit`'s scan and fails when
  `docs/design-system/kit-index.{md,json}` is stale.
- After adding, removing or changing a public component, run
  `cargo run -p rv2-kit -- index --write` and commit both files.
- A new public component needs a row in the Instructions table of
  `docs/architecture/003-ui-semantic-isa.md`, and an external consumer named
  there if no app here uses it (UI8). The index build fails without the row.
- Give it a `/kit` fixture in `apps/web/src/kit_*.rs`: that is where its
  example in the index comes from.
