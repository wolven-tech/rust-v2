# A2UI surfaces

**Status:** Accepted, and implemented in `crates/rv2-a2ui`,
`crates/rv2-a2ui-dioxus`, `apps/api` (`/a2ui/*`) and `apps/app` (`/runway`).

This extends, without replacing,
[`003-ui-semantic-isa.md`](003-ui-semantic-isa.md) and
[`002-generative-ai-contract.md`](002-generative-ai-contract.md).

[A2UI](https://a2ui.org) lets an agent describe UI as data: a flat list of
components picked from a catalog the client trusts, a JSON data model they bind
to, and named actions the client sends back. The client owns the rendering.
That is the same split the semantic ISA already enforces between an app and
`rv2-ui`, so an A2UI catalog here is a published subset of the ISA, not a
second component set.

The wire version is **v0.9.1**. v1.0 is a release candidate and renames the
binding keys (`path` → `@path`), so it is a new catalog and a new parser, not
an edit to this one.

## What works end to end

A signed-in user opens `/runway` in `apps/app`. The page asks `apps/api` for the
`runway` surface, renders the stream it gets back, takes cash and monthly burn
through bound fields, and sends a `calculate` action. The API answers with a
data-model update and new components, and the page shows months of runway, the
figures behind it and a verdict. No model is involved: the agent is a
deterministic function in `apps/api`, which is the seam a model-backed agent
would replace.

## Decisions

| # | Decision | Why |
|---|---|---|
| A2UI1 | `rv2-a2ui` holds the wire types, the data model, the catalog and surface state. It is WASM-safe and depends on `serde`, `serde_json` and `thiserror` only. | The API emits and the app parses the same types, and both sides of the WASM boundary can hold them. |
| A2UI2 | The catalog is `rv2:a2ui/catalog/v1`, our own, not A2UI's basic catalog. Its components are ISA instructions under their kit names, with operands as lowercase strings. | The basic catalog's `Row`, `Modal`, `Image` and the rest have no ISA instruction, and inventing them for the protocol's sake is what UI8 forbids. |
| A2UI3 | The catalog's enums are wire types in `rv2-a2ui`, mapped to `rv2-ui` operands by an exhaustive `match` in the renderer. `rv2-ui` gains no serde. | The catalog is a published contract that only grows; the kit's operands may be renamed. Keeping them apart lets the kit stay `dioxus`-only. |
| A2UI4 | The catalog only grows. A prop or a component is added, never renamed or removed; anything else is `v2`. `crates/rv2-a2ui/catalog/rv2-v1.json` is generated from the same table the parser checks, and a test fails when the committed file differs. `catalog/released/v1.json` is the catalog as first published and is never regenerated; `tests/released.rs` fails when the live catalog drops a component, a prop or an operand, newly requires a prop, or narrows a range. | An agent built against the published document must keep working, the same rule events follow (D-numbers in 001). |
| A2UI5 | A surface is untrusted input. A message is applied atomically: unknown component, unknown prop, wrong type, a second `root`, a cycle, depth over 16 or more than 256 components rejects the whole message and leaves the surface as it was. | Same stance as AI3: what a model produced is checked before anyone sees it. A half-applied update is a page nobody designed. |
| A2UI6 | The agent never sets `class`, colour or layout utilities. Layout is `Column`, `Grid` and `Card`. | UI5 already says `class` is layout only; an agent-supplied class string would also be an injection path into the page's styles. |
| A2UI7 | Transport is REST with JSONL bodies: `GET /a2ui/surfaces/{name}` opens a surface, `POST /a2ui/actions` takes one action and answers with the messages it produced, `POST /a2ui/errors` takes a client validation error, `GET /a2ui/catalog` returns the catalog's JSON Schema. All four require a session and sit behind the rate limiter. | The spec allows any ordered, framed transport. REST keeps the API stateless, which `rv2-jobs` already assumes of every instance, and `gloo-net` can read it without a streaming client. |
| A2UI8 | An action carries every value the agent needs in its `context`, resolved by the client from the data model. The API keeps no surface state. | Same reason as A2UI7, and it is what v0.9.1 does: the data model reaches the server only through an action. |
| A2UI9 | Supported value forms are literals and absolute `{"path": "/…"}` bindings. Function calls, relative paths and child-list templates are rejected with a named error. | They need a scope and a function library that no consumer asks for yet. Rejecting them by name tells an agent author what is missing rather than rendering nothing. |

## The v1 catalog

| Component | Props | Instruction |
|---|---|---|
| `Column` | `children` | stacked `div` (layout only) |
| `Grid` | `children`, `columns` 1–4 | `Grid` |
| `Card` | `children`, `pad` (`none`/`default`/`roomy`), `surface` (`flat`/`slab`) | `Card` |
| `Heading` | `text`, `level` 1–4, `size` (`display`/`section`/`title`/`card`) | `Heading` |
| `Text` | `text`, `tone` (`default`/`muted`/`small`/`lead`) | `Text` |
| `Callout` | `text`, `title`, `verdict` (`note`/`clear`/`caution`/`stop`) | `Callout` |
| `Stamp` | `text`, `verdict` | `Stamp` |
| `StatStrip` | `stats` (`label`, `value`, `note`, `lean`), `density` (`compact`/`hero`) | `StatStrip` |
| `ResultPanel` | `label`, `value`, `context`, `lean` (`neutral`/`good`/`close`/`bad`) | `ResultPanel` |
| `Meter` | `label`, `value`, `max`, `readout`, `warnAt`, `criticalAt` | `Meter` |
| `TextField` | `label`, `value` (path), `placeholder`, `help`, `inputType` (`text`/`email`/`number`) | `TextField` |
| `UnitField` | `label`, `value` (path), `prefix`, `suffix`, `help` | `UnitField` |
| `Button` | `label`, `variant` (`primary`/`secondary`/`outline`/`danger`/`ghost`), `action` | `Button` |

Text and number props take a literal or a path. Input `value` must be a path:
a field the user types into writes to the data model, and a literal has nowhere
to write. `Button.action` is A2UI's `{"event": {"name", "context"}}`.

`Tone::OnInk`, `Variant::Link` and the overlay, motion and tactile modules are
not in v1: none of them is meaningful on a surface an agent places inside an
app page.

## Adding an agent

1. Pick components with `cargo run -p rv2-isa -- catalog`; `show <Name>`
   prints an entry's wire props next to the instruction it renders.
2. Write the agent in `apps/api/src/presentation/agents/` as `open()` and
   `act(&ActionMessage)`, building messages with `ComponentDef` and
   `ServerMessage`. `runway.rs` is the worked example.
3. Route its surface id in `presentation/handlers/a2ui.rs`.
4. Test it the way the client applies it: every stream the agent emits goes
   through `rv2_a2ui::testing::apply_as_client`, as
   `apps/api/tests/a2ui_runway.rs` does. That catches a component the catalog
   refuses and a surface left with no `root`.
5. Mount `A2uiSurface` on a page in `apps/app`, then open the page. CI does
   not render one.

## Adding to the catalog

A new component or prop is additive and stays in v1:

1. A row in `COMPONENTS` and a parse arm in `Node::parse`
   (`crates/rv2-a2ui/src/catalog.rs`). The component must be an instruction in
   003, or `rv2-isa index --check` fails.
2. A render arm in `crates/rv2-a2ui-dioxus`, mapping each wire operand to the
   kit's with an exhaustive `match`.
3. `RV2_BLESS_CATALOG=1 cargo test -p rv2-a2ui --test catalog`, then review
   the diff to `catalog/rv2-v1.json`.
4. A row in the v1 table above, and `cargo run -p rv2-isa -- index --write`.

Never edit `catalog/released/v1.json`. Removing or renaming anything is v2:
a new catalog id, a new released file, and a renderer that accepts both.

## Not built

- **A model-backed agent.** 001 lists the bundled model as deliberately not
  built, and 002 keeps the runtime product-owned. A product that wants one
  implements it behind the same handler and still answers in this catalog.
- **Server-sent streaming.** A whole response is a handful of lines. When an
  agent streams tokens, `/a2ui/surfaces` becomes SSE; the parser already reads
  one message per line.
- **v1.0.** See the version note above.
- **`sendDataModel`.** Accepted on `createSurface` and ignored: A2UI8 carries
  the values in the action instead.
