---
name: rv2-a2ui
description: Build or change an agent-placed A2UI surface in rust-v2 (crates/rv2-a2ui, rv2-a2ui-dioxus, apps/api/src/presentation/agents, /a2ui routes). Use when adding an agent or a surface, when an agent's output is refused ("unknown component", "has no prop", VALIDATION_FAILED in the logs), when adding a component or prop to the catalog, or when asked what an agent may place.
---

# Agent surfaces on the rv2 catalog

An agent describes UI as data: A2UI v0.9.1 messages naming components from
`rv2:a2ui/catalog/v1`. The client checks each message against the catalog and
renders it with `rv2-ui`. The catalog is a published subset of the instruction
set, so an agent can only place what the kit already offers, and it never sets
`class`, colour or layout utilities (A2UI6). Decisions A2UI1–A2UI9 are in
`docs/architecture/004-a2ui-surface.md`.

## What an agent may place

```sh
cargo run -p rv2-isa -- catalog          # every entry and the instruction it renders
cargo run -p rv2-isa -- show Callout     # the instruction, then its wire props
```

The JSON Schema an agent author or a model reads is
`crates/rv2-a2ui/catalog/rv2-v1.json`, and a running API serves the same
document at `GET /a2ui/catalog`.

Values are literals or absolute `{"path": "/…"}` bindings. Function calls,
relative paths and child-list templates are refused by name (A2UI9). An
input's `value` must be a binding.

## Adding an agent

1. Write it in `apps/api/src/presentation/agents/<name>.rs` as `open()` and
   `act(&ActionMessage)`. `runway.rs` is the worked example: one surface id,
   one action name, a data-model update for figures, components for structure.
2. Route its surface id in `apps/api/src/presentation/handlers/a2ui.rs`.
3. Put everything the agent needs in the action's `context`; the API keeps no
   surface state (A2UI8).
4. Test every stream it emits with `rv2_a2ui::testing::apply_as_client`, as
   `apps/api/tests/a2ui_runway.rs` does. It frames the messages as JSONL,
   parses them back and applies them as the client does, and fails on a
   refused component or a surface left without `root`.
5. Mount `A2uiSurface` in `apps/app` (see `apps/app/src/runway.rs`), run
   `dx serve --package app --platform web`, and open the page. CI renders
   nothing.

## Adding a component or a prop

The catalog only grows (A2UI4). Follow "Adding to the catalog" in 004:

1. A `COMPONENTS` row and a `Node::parse` arm in
   `crates/rv2-a2ui/src/catalog.rs`. It must name an `rv2-ui` instruction.
2. A render arm in `crates/rv2-a2ui-dioxus/src/lib.rs`.
3. `RV2_BLESS_CATALOG=1 cargo test -p rv2-a2ui --test catalog`, and review
   the diff to `rv2-v1.json`.
4. A row in 004's v1 table and `cargo run -p rv2-isa -- index --write`.

Never edit `crates/rv2-a2ui/catalog/released/v1.json`. If
`tests/released.rs` fails, the change removed, renamed or narrowed something
v1 published: make it additive, or it is v2.

## When the kit changes

An instruction the catalog places has a consumer under UI8 and cannot be
deleted while v1 places it. `rv2-isa index --check` names the entry. Renaming a
kit operand is fine: the renderer's exhaustive `match` breaks at compile time,
and the wire value stays as published.
