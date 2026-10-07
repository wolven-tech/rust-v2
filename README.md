<div align="center">

# rust-v2

**All-Rust foundation with AllSource as the sole datastore.**<br>
One language, one event-sourcing database, no TypeScript in the data path. The successor to rust-v1 and the foundation several product repositories copy from.

[![Rust](https://img.shields.io/badge/Rust-2024-000000?logo=rust&logoColor=white)](rust-toolchain.toml)
[![CI](https://github.com/wolven-tech/rust-v2/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/wolven-tech/rust-v2/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.97.1-informational)](rust-toolchain.toml)

[![macOS](https://img.shields.io/badge/AllSource-native_binary-FF6B35?logo=apple&logoColor=white)](#run)
[![Linux](https://img.shields.io/badge/AllSource-docker__compose-FCC624?logo=linux&logoColor=black)](#run)
[![Dioxus](https://img.shields.io/badge/Frontend-Dioxus_CSR-000000)](apps/app)

[![Built with Rust, for multi-platform event-sourcing with AllSource and Dioxus UIs](https://skillicons.dev/icons?i=rust,dioxus,docker,githubactions)](https://skillicons.dev)

[GitHub](https://github.com/wolven-tech/rust-v2) · [Architecture](docs/architecture/001-rust-v2-allsource-foundation.md) · [Contributing](#contributing) · [Docs](#docs)

</div>

---

**rust-v2 is a complete workspace for building event-sourced applications.**
It powers `apps/api` (Axum HTTP server), `apps/app` (authenticated Dioxus SPA), and
`apps/web` (SSG marketing site), all sharing a single crate graph.

## Prerequisites

| | |
|---|---|
| **Rust** | `1.97.1` — pinned in `rust-toolchain.toml` via `rustup` |
| **WASM target** | `wasm32-unknown-unknown` — declared in `rust-toolchain.toml` |
| **Task runner** | `cargo install monorepo-meta` — orchestrates `meta dev`, `meta ci` |
| **Frontend CLI** | `cargo install dioxus-cli --version 0.7.10 --locked` — builds and serves Dioxus apps |
| **Build monitor** | `cargo install bacon` — used by `meta dev` to watch the API |
| **AllSource Core** | Local binary or Docker (see below) |

### Setting up AllSource Core

**Native binary (recommended for local development):**

```bash
cargo install allsource-core --version 0.23.0 --locked
ALLSOURCE_HOST=0.0.0.0 ALLSOURCE_PORT=3900 ALLSOURCE_DATA_DIR=.allsource-data ALLSOURCE_DEV_MODE=true allsource-core
```

Verify: `curl -s http://localhost:3900/health`

**Docker (community images only):**

```bash
docker-compose up
# Pulls ghcr.io/all-source-os/allsource-core-community:latest
```

Note: community images are `linux/amd64` only; Apple Silicon runs under emulation.

## Run

```bash
git clone git@github.com:wolven-tech/rust-v2.git && cd rust-v2
cp .env.example .env
set -a && source .env && set +a
meta dev                 # tmux: Core + api (bacon) + app (dx :4402) + web (dx :4401)
```

Or individually:

```bash
cargo xtask core                                  # Terminal 1: AllSource on :3900
cargo run -p api --features allsource-auth       # Terminal 2: API on :4400
dx serve --package app --platform web --port 4402  # Terminal 3: dashboard on :4402
```

After editing Tailwind classes, recompile: `cargo xtask styles`. The compiled CSS is committed for correct rendering on a fresh clone.

| Service | Port |
|---|---|
| AllSource Core | 3900 |
| AllSource Query Service (optional) | 3902 |
| `apps/api` | 4400 |
| `apps/web` (marketing) | 4401 |
| `apps/app` (dashboard) | 4402 |

## What it does

| | |
|---|---|
| **API** | `apps/api`: Axum HTTP server, the only process in production. Mounted with `better-auth` for OAuth and credential auth. The only event writer. |
| **Dashboard** | `apps/app`: Dioxus CSR SPA, 100% WASM-compiled (`wasm32-unknown-unknown`), reads events through the HTTP API with authenticated cookies |
| **Marketing site** | `apps/web`: Dioxus fullstack app built as SSG at compile time, deployed as static HTML without a running process |
| **Event store** | AllSource Core on `:3900` + Query Service (optional, gateway only). Reads are tenant-scoped; writes are never, because Core handles multi-tenancy |
| **Shared crates** | Events, domain logic, DTOs, UI kit, HTTP client, AllSource integration, analytics, email, background jobs — layered so WASM crates never depend on server-only ones |

## Architecture

21 numbered design decisions (`D1`–`D21`), risks, and open questions live in
[`docs/architecture/001-rust-v2-allsource-foundation.md`](docs/architecture/001-rust-v2-allsource-foundation.md).
Read it before changing anything structural; code comments cite these identifiers.

**Crate layers:**

1. `rv2-events` — domain event types and wire format
2. `rv2-domain`, `rv2-api-types` — pure logic and DTOs
3. `rv2-ui`, `rv2-client`, `rv2-ai` — WASM-safe, no I/O
4. `rv2-allsource`, `rv2-shared`, `rv2-analytics`, `rv2-email`, `rv2-jobs` — server-only
5. `apps/*` — leaves; HTTP handlers read/write events

**The WASM boundary is enforced by cross-compile.** `cargo xtask ci` proves
`wasm32-unknown-unknown` compiles for every WASM crate and **fails** if a server-only
dependency (like `tokio` with networking) appears in the app build.

**Events are immutable forever.** Fields may only be added (`#[serde(default)]`).
A breaking change becomes a new event type; old variants stay in code permanently.
`crates/rv2-events/tests/golden/` holds one captured JSON payload per schema
version and every one must continue deserializing.

**`vendor/better-auth` preserves the published 0.10.0 library source.** Its
manifest removes an unused production YAML dependency and omits upstream
maintenance tools. [Provenance and removal condition](vendor/better-auth/PROVENANCE.md).

## Test

```bash
cargo xtask ci        # whole gate: rustfmt, clippy, build, test, styles, WASM boundary
cargo xtask live      # tests that need a live AllSource Core (marked #[ignore])
```

The WASM boundary is asserted by real cross-compile in both directions,
not by static inspection.

## Deploy

```bash
docker build -t rust-v2-api .
docker run --rm -p 4400:4400 --env-file .env rust-v2-api
```

The Dockerfile builds `apps/api` only. `apps/web` and `apps/app` have separate
release boundaries validated by `cargo xtask stage-web` and `cargo xtask stage-app`.
Both require product-specific `PUBLIC_*` values set in `.env`.

Two probes are required in production:

| Probe | Endpoint | Answers |
|---|---|---|
| Liveness | `GET /health` | Is the process responsive? Restart if no. |
| Readiness | `GET /ready` | Should this instance receive traffic? Reports dependency health. |

`/health` checks **nothing** beyond process responsiveness. A liveness probe that checks AllSource kills every instance during a Core outage; `/ready` is where dependency checks belong.

## Known gaps and limitations

| Gap | Where | Why |
|---|---|---|
| Google OAuth not wired | `apps/api/src/infrastructure/auth/better.rs` | Needs HMAC-signed pending-origin cookie glue to avoid open redirect. Credential auth is complete. |
| No session cache | `apps/api/src/infrastructure/auth/middleware.rs` | Authenticated requests cost two AllSource round-trips. Measure p99 before caching. |
| No durable job queue | `crates/rv2-jobs` | Periodic in-process only. Every instance runs every job; nothing survives a restart. See the crate docs before adding a job. |
| Rate limits are per-instance | `apps/api/src/infrastructure/rate_limit.rs` | In-memory. N instances allow N× the limit; restarts clear them. Correct for the current load. |
| No browser UI tests | — | `cargo xtask live` drives the API with `reqwest` and JSON. No page ever opens in CI. Manual sign-in remains the final verification. |
| English only | — | No i18n crate evaluated. rust-v1 shipped English and French; this is a regression needing sign-off. |

**Tenant scoping is mandatory in reads.** AllSource answers a query with no `tenant_id` with HTTP 200 and empty results—no error. A read path that omits it silently returns nothing forever. Use `crates/rv2-allsource/src/tenant_query.rs`.

**New HTTP dependencies default to the wrong TLS.** `deny.toml` bans `native-tls` and `openssl-sys`. Assume the next dependency defaults to native TLS and needs `default-features = false` plus an explicit `rustls` feature.

## Observability (opt-in)

Logging, metrics, and traces all start **off** by default, so a fresh clone runs with
zero external dependencies. Each remains **cost-free when unconfigured:**

```bash
LOG_FORMAT=json cargo run -p api              # structured JSON logging (off: human-readable)
METRICS_ADDR=0.0.0.0:9090 cargo run -p api   # Prometheus on separate port (off: no recorder)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo run -p api  # OTLP tracing (off: no exporter)
```

HTTP metrics are labeled by matched route (`/posts/{id}`), never the request path.

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) for the rules. The gate is `cargo xtask ci`.

This repo follows portable rules published at [decebal-claude-skills](https://github.com/decebal/decebal-claude-skills) → `rules/`:

- `git-discipline`: One PR = one commit, rebase-merged
- `evidence-discipline`: Check the destination before trusting an absence
- `definition-of-done`: End-to-end or not done
- `comments`: Say *why*, not what

## Docs

| Document | What it is |
|---|---|
| [`docs/architecture/001-rust-v2-allsource-foundation.md`](docs/architecture/001-rust-v2-allsource-foundation.md) | 21 design decisions (D1–D21), risks (R*), open questions (OQ-*). Code comments cite these identifiers. |
| [`docs/architecture/002-generative-ai-contract.md`](docs/architecture/002-generative-ai-contract.md) | Grounded-draft and AI runtime boundaries (AI1–AI5). |
| [`docs/architecture/003-ui-semantic-isa.md`](docs/architecture/003-ui-semantic-isa.md) | The UI kit as a closed vocabulary — registers, operands, instructions (UI1–UI9) — and the `cargo xtask ci` check that keeps apps on it. |
| [`docs/ledger/allsource-integration-corpus.md`](docs/ledger/allsource-integration-corpus.md) | 21 AllSource behaviours this codebase depends on (B1–B21), which are asserted in tests. |
| [`docs/review/001-bootstrap-readiness.md`](docs/review/001-bootstrap-readiness.md) | An external review of rust-v2 as a foundation; twelve findings and what was done about each. |
| [`docs/design-system/field-and-action-contracts.md`](docs/design-system/field-and-action-contracts.md) | Shared field labels, help/error associations, typed-control composition, busy actions and verification limits. |
| [`docs/plans/2026-10-12-command-select-design.md`](docs/plans/2026-10-12-command-select-design.md) | Shared searchable selection, consumption API, theme variables and keyboard/browser acceptance. |
| [`docs/NEXT_BET_START.md`](docs/NEXT_BET_START.md) | Checklist for placing a new product bet from this foundation. |
| [`AGENTS.md`](AGENTS.md) | Short orientation for anyone arriving with no context. Read this first. |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | The rules and what enforces each one. |
| [`SECURITY.md`](SECURITY.md) | How to report a vulnerability. |

## License

MIT
