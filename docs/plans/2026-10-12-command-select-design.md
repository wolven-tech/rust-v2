# Shared command-select

`rv2_ui::CommandSelect` is a controlled, renderer-neutral single-select for
workspace pickers and other searchable choices. A Place Ahead consumes the same
source in its foundation copy. Each bet supplies stable `(value, label)` pairs
and an `onchange: EventHandler<String>`; the component owns filtering, focus and
presentation. Selecting the current value emits no change. Searching, cancelling
or closing the popup cannot change the selected value.

## Consumption

Copy `crates/rv2-ui/src/command_select.rs` and
`crates/rv2-ui/assets/command-select.css` with the foundation crate, export
`CommandSelect` from its `lib.rs`, and enable Dioxus `mounted` alongside `lib`.
`mounted` enables focus operations on an existing renderer without choosing one.
The component bundles its stylesheet through `asset!`; app styles do not need
Tailwind regeneration or a separate CSS import. It adds no runtime dependency.

```rust,ignore
CommandSelect {
    id: "workspace-picker", label: "Your workspace", value: selected(),
    options: workspaces.iter().map(|row| (row.id.clone(), row.name.clone())).collect::<Vec<_>>(),
    onchange: move |id| selected.set(id),
}
```

`id` must be unique within the document. Optional `name` emits a hidden form
input. Optional `disabled` disables selection and form submission. Placeholder,
search placeholder and empty-message strings are caller-owned. `class` allows a
product scope. Theme through `--rv2-command-paper`, `--rv2-command-ink`,
`--rv2-command-muted`, `--rv2-command-rule`, `--rv2-command-active` and
`--rv2-command-focus` while retaining adequate contrast. Each defaults to the
matching kit register (`--color-surface`, `--color-ink`, `--color-ink-muted`,
`--color-rule`, `--color-raised`, `--color-accent`), falling back to the
original palette where no theme defines them, so the picker follows a dark
theme without overrides.

## Interaction and accessibility

The labelled button opens a nonmodal popup containing a searchable ARIA
combobox and listbox. Focus moves to search on open. Arrow keys wrap through
filtered results; Home/End move to first/last result; Enter commits; Escape
cancels and returns focus. Search requires every typed word in the visible
label, ignoring case. Pointer selection retains input focus until commit.
Tab or an outside focus change dismisses the popup. IME composition cannot
commit an option. Selected values and active options have separate indicators.
All action targets are at least 44 CSS pixels, with bounded scrolling and
reduced-motion/forced-colour support.

The shared component never loads workspaces, navigates, authorises access or
persists selections. Those behaviors remain in each bet's application layer.
Callers should avoid clipping ancestors around an anchored popup.

## Verification

- `cargo test -p rv2-ui --test command_select`: matching, navigation and rendered
  closed-state semantics, hidden form value, disabled state.
- `dx serve --package app --example command_select --platform web --port 4419`:
  synthetic preview for keyboard, pointer, empty/disabled choices, long lists,
  independent instances and mobile layout.
- Verify the consuming app's workspace data path and renderer build separately;
  the preview does not establish account switching or production deployment.

The A Place Ahead release build and its existing selected-ID API path were
verified against two synthetic, write-disabled workspaces. At a 320px viewport,
the app document remained 320px wide and the popup stayed between x=18 and x=302.
No private accounts were exported and the UI has not been deployed.

The lockfile uses `rustls` 0.23.45, which addresses `RUSTSEC-2026-0285`.
The [better-auth library patch](../../vendor/better-auth/PROVENANCE.md) removes
unused production YAML dependencies while preserving all six upstream source
files. The full dependency audit passes without a YAML advisory exemption.
Source checksums and lockfile regression tests enforce that constraint. The
component's SSR test reuses the existing `dioxus-ssr` version.

`CLAUDE.md` points to `AGENTS.md` through a valid relative symlink, so GitHub
runners can check out the repository and read the same agent instructions.
