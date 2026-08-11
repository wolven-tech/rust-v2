# Frozen corpus — AllSource behaviours rust-v2 depends on

**Frozen 2026-08-11. Do not add or remove rows while the loop is running.** The list is what
the scalar is measured against; editing it mid-loop lets the loop "improve" by listing fewer
behaviours, which is the one way this measurement can be gamed.

Derived from every AllSource interaction point in `crates/rv2-allsource`,
`crates/better-auth-allsource` and `apps/api` — not from a wish list. Each row is a behaviour
the code would break without.

**Asserted** means *an automated test fails if the behaviour changes*. A behaviour observed once
by a human during development is **not** asserted: three of the four defects found this session
were in code carrying a comment describing the correct behaviour. A comment does not survive a
dependency bump.

| # | Behaviour | Depended on by | Asserted? |
|---|---|---|---|
| B1 | Append returns `event_id` + monotonic `version` | `writer.rs` | ✓ `b1` |
| B2 | Append preserves `metadata` alongside `payload` | `writer.rs::append_with_metadata` | ✓ `b2` |
| B3 | The SDK rewrites the event type through `normalize_event_type` on ingest, unconditionally — **client-side; Core itself returns 400** | `writer.rs` (guarded per-append) | ✓ `b3` |
| B4 | `<domain>.<entity>.<action>` survives normalization unchanged (our grammar is a fixed point) | D10/D11, all event types | ✓ `b4` |
| B5 | **A query without `tenant_id` returns `{"events":[],"count":0}` and HTTP 200 — not an error** | `tenant_query.rs`, `better-auth-allsource` | ✓ `b5` |
| B6 | A query *with* `tenant_id` returns the entity's events | `tenant_query.rs` | ✓ `b5` |
| B7 | Query results are ordered OLDEST-first | `latest_live_payload`, all folders | ✓ `b7` |
| B8 | `entity_id` filters to exactly that entity | `tenant_query.rs` | ✓ `b8` |
| B9 | `event_type_prefix` filters to that family | `better-auth-allsource` | ✓ `b9` |
| B10 | `payload_filter` matches top-level payload fields | `better-auth-allsource::find_by_field` | ✓ `b10` |
| B11 | An appended event is queryable immediately (read-after-write) | `posts::create` read-back | ✓ `b11` |
| B12 | A `_deleted: true` tombstone is the latest event and means "gone" | `latest_live_payload` | ✓ `b12` |
| B13 | `ProjectionWorker` streams events over WebSocket | `workers.rs` | ✗ |
| B14 | **`ProjectionWorker::start` does NOT hydrate — it streams from the cursor into `Default`** | `workers.rs` hydrate-on-boot | ✗ |
| B15 | `state_flush_entities` persists per-entity state into Core's projection KV | `workers.rs` | ✓ `b15_b16` |
| B16 | `get_projection_state_summary` reads that state back | `workers.rs` boot hydrate | ✓ `b15_b16` |
| B17 | Renaming the durable consumer id replays the stream from zero | D15 (rebuild strategy) | ✓ `b17` |
| B18 | The worker receives only its declared event families | `workers.rs` | unit-only |
| B19 | Sessions-as-events: the latest payload wins over the `*.created` one | `better-auth-allsource` | indirect |
| B20 | `QueryEventsParams` cannot express `tenant_id` (SDK limitation, v0.23.0) | why `tenant_query` exists | ✗ |
| B21 | Optimistic concurrency (`expected_version`) is unavailable via the SDK | OQ-2, no OCC anywhere | ✗ |

**Legend.** `✗` = no automated assertion. `indirect` = exercised by the end-to-end vertical
slice, so a change would break *a* test, but the failure would point at an HTTP handler rather
than at the behaviour. `unit-only` = asserted without a live Core, so it tests our declaration
rather than Core's behaviour.

## Scalar

**Behaviours with no direct automated assertion against a live Core.** Lower is better.

`indirect` and `unit-only` both count as unasserted: the whole lesson of the tenant defect is
that an end-to-end test which happens to pass tells you nothing about *which* assumption holds.
`GET /posts` passed throughout that bug because it used a different code path.

## Baseline

| | |
|---|---|
| Behaviours in corpus | 21 |
| Directly asserted against a live Core | 0 |
| **Unasserted (the scalar)** | **21** |

---

# Ledger

Loop shape: [karpathy/autoresearch](https://github.com/karpathy/autoresearch) — propose one
change, score on a single scalar, keep or discard, append a row.

**Scalar** — corpus behaviours with no direct automated assertion against a live Core. Lower is
better. Deterministic (a count), so neither statistical guard applies.

**Gate** — `cargo build/test/clippy -D warnings/fmt --check/deny check/machete`, the wasm32
cross-compile, and all three vertical-slice tests against a live Core.

**Stop condition** — 2 consecutive discards, or effort budget.

| # | Proposal | Scalar | Δ vs baseline | Verdict |
|---|---|---|---|---|
| 0 | Baseline | 21 | — | — |
| 1 | Assert the read/query contract (B1, B5–B12) in `tests/core_contract.rs`, and run it in CI | 12 | −9 (−43%) | **Keep** — the class that produced the tenant defect |
| 2 | Assert the projection-KV round-trip and rebuild semantics (B15, B16, B17) | 9 | −12 (−57%) | **Keep** — the mechanism the worker-restart fix depends on, previously unasserted |
| 3 | Assert the write contract (B2, B3, B4) | **6** | **−15 (−71%)** | **Keep** — and it corrected a wrong belief, below |

**Net: 21 → 6 unasserted (−71%). 13 contract tests, all running in CI.**

Stopped on effort budget, not dryness — the stop condition was never reached.

## The gate rejected a proposal, and the rejection was the finding

Proposal 3's first draft asserted that Core normalizes a PascalCase event type. It failed:

```
append failed: 400
```

**Core does not normalize — it rejects.** Posting `ContractProbeCreated` over raw HTTP returns
400. The normalization is entirely **client-side**: `CoreClient::ingest_event` rewrites
`input.event_type` before posting.

The belief being tested was wrong, not the code. That distinction is worth keeping:

- An event written by any non-SDK client — `curl`, another language's SDK, a tool that bypasses
  `EventWriter` — **fails loudly** rather than landing under a mangled name. That is a stronger
  guarantee than D11 assumed, and it is now asserted.
- `b3` had to move from raw HTTP onto the SDK to test the thing it claimed to test. A test can
  be green and still be pointed at the wrong layer.

The counter-test matters as much as the test: without `b3` proving the normalizer is *active*,
`b4`'s fixed-point assertion would still pass if normalization were removed entirely — and would
then be asserting nothing.

## Why an end-to-end test was not enough

Every AllSource defect found in this codebase was documented in a comment and asserted nowhere,
and in every case the end-to-end suite stayed green:

| Defect | What the e2e suite did |
|---|---|
| Tenant-less query returns empty, not an error | `GET /posts` passed throughout — it uses the worker, not the query path |
| `ProjectionWorker` does not hydrate on restart | Never restarted a worker |
| SDK rewrites event types on ingest | Grammar happened to be a fixed point |

A green end-to-end test tells you the feature works on the path it took. It says nothing about
which assumption underneath it holds. These tests fail with the name of the broken behaviour
instead of a confusing 404.

## Still unasserted (6)

| # | Behaviour | Why not yet |
|---|---|---|
| B13 | Worker streams over WebSocket | Needs a running worker and a live socket; heavier than the HTTP assertions here |
| B14 | `ProjectionWorker::start` does not hydrate | Needs a genuine process restart to prove, not just a fresh handle |
| B18 | Worker receives only its declared families | Currently unit-only — asserts our declaration, not Core's routing |
| B19 | Sessions-as-events: latest payload wins | Indirect via the slice; deserves a direct test given `.first()` vs `.last()` already regressed once |
| B20 | `QueryEventsParams` cannot express `tenant_id` | A compile-time fact about the SDK; B5 documents the consequence |
| B21 | No optimistic concurrency (`expected_version`) | An absence. Asserting it means asserting a field does not exist |

B13, B14 and B18 are one cluster: they all need a worker harness that can start, stop and
restart against a live Core. That is the obvious next proposal.

## Found but not added — the corpus is frozen

**Core rejects malformed event types with HTTP 400.** Discovered by proposal 3's failure. It is
a real, protective behaviour worth asserting, but adding a row mid-loop would let the loop
change its own denominator. It belongs in the next round's corpus.
