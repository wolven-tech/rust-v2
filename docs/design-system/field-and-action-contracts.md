# Shared field and action contracts

`rv2-ui` owns label, help, validation and busy-action semantics. Product code owns
parsing, domain validation, request state, error wording and persistence. These
contracts are additive; existing `TextField`, `TextArea` and `Button` call sites
continue to compile without the new props.

## Text controls

`TextField` and `TextArea` accept `id`, `help`, `error` and `disabled`.

- A visible label uses `for` to name the control.
- Nonempty help and error messages receive `<id>-help` and `<id>-error` IDs.
  `aria-describedby` includes only IDs whose messages are rendered, in that order.
- A nonempty error sets `aria-invalid=true` and renders an alert. Missing or
  whitespace-only errors leave the field valid. A product must supply its actual
  validation result; absence of an error does not establish domain correctness.
- `disabled` uses the native attribute. Required, name, autocomplete and type
  props on `TextField` retain their existing behavior.
- An omitted ID is generated from the component's Dioxus scope. Supply a stable,
  unique explicit ID for external label references or multiple application roots.

```rust,ignore
TextField {
    id: "purchase-price",
    label: "Purchase price",
    value: price_text(),
    help: "Enter pounds. Leave blank when unknown.",
    error: price_error,
    disabled: saving,
    oninput: move |event| price_text.set(event.value()),
}
```

## Custom typed controls

`FieldShell` provides the visible label and message nodes. Its caller must attach
the same `id` to exactly one labelable control and bind the returned description
and invalid state. The shell cannot add attributes to an arbitrary child. Use
`field_description` to keep the rendered messages and references synchronized.

```rust,ignore
let described_by = field_description("amount", help.as_deref(), error.as_deref());
let invalid = error.as_ref().is_some_and(|text| !text.trim().is_empty());
rsx! {
    FieldShell { id: "amount", label: "Works budget", help, error,
        input {
            id: "amount",
            inputmode: "decimal",
            "aria-describedby": described_by,
            "aria-invalid": invalid,
            value: amount_text(),
            oninput: move |event| amount_text.set(event.value()),
        }
    }
}
```

Keep money parsing in the product domain. A blank value and an explicit zero
have different meanings and must not be collapsed by presentation code.

## Busy actions

`Button { busy: true }` sets `aria-busy=true`, disables the native button, and
guards its handler against activation. Explicit `disabled` also suppresses the
handler. `busy=false` does not override an explicit disabled state. Button type
continues to default to `button`, so adding a busy state does not introduce form
submission.

Supply a meaningful child label such as “Saving comparison” while the owning
request is in progress. The component does not infer request state or replace
its label. A disabled action that needs an explanation must have visible nearby
copy supplied by the product.

## Verification and limits

`crates/rv2-ui/tests/field_contracts.rs` checks rendered label bindings, help and
error references, generated-ID uniqueness within one Dioxus root, whitespace
messages, native disabled states and busy attributes. Run it with the existing
kit tests:

```text
cargo test -p rv2-ui
```

These server-rendered checks establish DOM semantics only. They do not establish
WCAG 2.2 AA conformance. Consuming products must verify keyboard interaction,
focus, live error announcements, contrast, reflow, target spacing, request
failure/retry behavior and complete user journeys in their actual browser build.
Contrast depends on the consuming theme; the component kit cannot certify a
product by itself. See [WCAG 2.2 conformance requirements](https://www.w3.org/TR/WCAG22/#conformance-reqs).

This change adds no dependency, product data, telemetry or network access.
