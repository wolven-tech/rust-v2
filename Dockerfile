# `apps/api` — the only server process in rust-v2.
#
# The repository had no container build at all, which meant it went from `git
# clone` to `localhost` and stopped. That is a fine starting point and a bad
# place to leave a foundation other projects are meant to be started from.
#
# Two stages: build with the full toolchain, ship a runtime image with none of
# it. What actually gets shipped is one statically-linked-ish binary and a CA
# bundle.
#
# NOT built here: the Dioxus frontends. `apps/app` and `apps/web` compile to
# wasm bundles via `dx bundle` and are static assets — they belong on a CDN or
# behind a static file server, not in the API's process. Putting them in this
# image would couple a frontend deploy to an API deploy for no reason.

# ── build ────────────────────────────────────────────────────────────────────
# `rust:1-slim-bookworm` rather than a pinned `rust:1.97.1`: rust-toolchain.toml
# pins the exact version, and rustup in the image honours it. Pinning here too
# would give two places to update and a confusing error when they disagree.
FROM rust:1-slim-bookworm AS builder

# `pkg-config`/`libssl-dev` are deliberately ABSENT. Everything in this
# workspace is rustls — deny.toml bans `native-tls` and `openssl-sys` outright —
# so needing them here would mean the ban has been breached.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Manifests first, so the dependency layer is cached against source edits. The
# whole workspace's manifests are needed because a member missing from the graph
# changes resolution.
COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY apps/api/Cargo.toml apps/api/
COPY apps/app/Cargo.toml apps/app/
COPY apps/web/Cargo.toml apps/web/
COPY crates/better-auth-allsource/Cargo.toml crates/better-auth-allsource/
COPY crates/rv2-allsource/Cargo.toml crates/rv2-allsource/
COPY crates/rv2-analytics/Cargo.toml crates/rv2-analytics/
COPY crates/rv2-api-types/Cargo.toml crates/rv2-api-types/
COPY crates/rv2-client/Cargo.toml crates/rv2-client/
COPY crates/rv2-domain/Cargo.toml crates/rv2-domain/
COPY crates/rv2-email/Cargo.toml crates/rv2-email/
COPY crates/rv2-events/Cargo.toml crates/rv2-events/
COPY crates/rv2-shared/Cargo.toml crates/rv2-shared/
COPY crates/rv2-ui/Cargo.toml crates/rv2-ui/
COPY tooling/xtask/Cargo.toml tooling/xtask/

# Stub sources so `cargo fetch` can resolve the graph. `fetch` rather than a
# stub `build`: it downloads every dependency into the layer without compiling
# anything that the real source would immediately invalidate.
RUN set -eux; \
    for dir in apps/api apps/app apps/web tooling/xtask; do \
      mkdir -p "$dir/src" && echo 'fn main() {}' > "$dir/src/main.rs"; \
    done; \
    for dir in crates/*; do \
      mkdir -p "$dir/src" && touch "$dir/src/lib.rs"; \
    done; \
    cargo fetch --locked

COPY . .

# `--locked` so the image can never resolve a different dependency set than the
# one CI checked. `--offline` because everything is already in the layer above;
# if it is not, that is a cache bug worth failing on rather than papering over
# with a network call.
RUN cargo build --release --locked --offline -p api --features allsource-auth

# ── runtime ──────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# `ca-certificates` is required, not optional: rustls verifies AllSource,
# PostHog and Resend against the system trust store, and without it every
# outbound HTTPS call fails with an opaque certificate error at runtime.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*

# Non-root. The process binds :4400, which needs no privilege.
RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin api
USER api

COPY --from=builder /build/target/release/api /usr/local/bin/api

ENV HOST=0.0.0.0 \
    PORT=4400 \
    LOG_FORMAT=json

EXPOSE 4400

# Liveness, matching the endpoint's own contract: `/health` checks no
# dependency, so this restarts the container only when the process itself is
# wedged. Readiness (`/ready`) is the orchestrator's business, not Docker's —
# Docker has no way to act on "route around me" and would restart on it.
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD curl -fsS "http://127.0.0.1:${PORT}/health" || exit 1

ENTRYPOINT ["/usr/local/bin/api"]
