# Discovery release foundation design

Status: approved by founder request on 2026-09-01.

## Goal

New product bets must begin with crawlable discovery, hardened static hosting,
and truthful acquisition verification instead of rebuilding those capabilities
after product work. Foundation remains product-neutral: no Talk Run Card copy,
domain, seller claim, price, or gate enters starter.

## Decision

Resolve D8/OQ-10 with verified Dioxus 0.7 SSG. `apps/web` exposes
`static_routes`, uses build-only server features, and emits complete HTML for
every route. Product-specific copy still replaces fixture content, but initial
response already has titles, descriptions, canonicals, social metadata, and
WebSite/WebPage structured data derived from compile-time public configuration.

`cargo xtask stage-web` becomes release boundary. It copies only generated
public output, rejects localhost or placeholder configuration, promotes
`robots.txt`, `sitemap.xml`, and `llms.txt` to root, and creates stable public
IndexNow ownership file. Same-origin social image URL is required so Open Graph
and Twitter previews cannot silently ship blank. Dioxus SSG always emits inline hydration bootstrap. If
generated routes contain no client event handlers, staging removes bootstrap,
module loader, and unused runtime. If any route is interactive, staging moves
inline executable scripts into content-addressed same-origin assets so CSP can
block inline script without breaking hydration. JSON-LD remains inline as a
non-executable data block.

`cargo xtask indexnow` verifies deployed ownership file before notifying global
endpoint. Search Console ownership, sitemap submission, URL inspection,
PageSpeed, and CrUX remain explicit launch checks. CrUX absence is unknown field
evidence, never failure or fabricated pass.

`cargo xtask stage-app` applies same production-identity rule to authenticated
CSR bundle. Product name, website origin, and API origin must be compiled into
WASM; localhost rejects staging. App brand links to public website, dashboard
keeps separate internal link, inline bootstrap scripts are externalised, and
app stays `noindex,nofollow,noarchive`.

## Deployment and failure behaviour

Pinned Nginx image serves ignored staged bundle with one-year asset caching,
no-cache HTML, UTF-8, privacy-minimised logs, HSTS, restrictive permissions,
frame denial, and CSP. Fly configuration remains template because app names and
owner organisation belong to each bet.

Staging fails when SSG output is absent, public URLs are not HTTPS, SEO
templates retain placeholders, structured data disappears, interactive HTML is
paired with stripped runtime, or generated runtime shape changes. IndexNow
fails when public key differs or endpoint rejects submission. No discovery
command changes commercial gate or counts indexing, clicks, or impressions as
customer evidence.

## Verification

- Unit tests cover static-runtime removal, interactive-runtime externalisation,
  placeholder replacement, and stable IndexNow key derivation.
- `cargo xtask ci` remains repository gate.
- Exact SSG build and `cargo xtask stage-web` must pass from clean output.
- Staged HTML must contain route copy before JavaScript and contain no localhost.
- Browser smoke covers `/`, `/about`, and interactive `/motion` fixture.
- Production checklist requires live headers, root discovery files, Google
  ownership/sitemap/inspection, mobile/desktop PageSpeed, and honest CrUX state.

## Source paths

- `apps/web/src/main.rs`
- `apps/web/index.html`
- `apps/web/assets/favicon.svg`
- `apps/app/src/lib.rs`
- `apps/app/src/views.rs`
- `apps/app/assets/favicon.svg`
- `tooling/xtask/src/main.rs`
- `deploy/nginx.static.conf`
- `deploy/nginx.spa.conf`
- `deploy/static.Dockerfile`
- `deploy/fly.web.toml.example`
- `deploy/fly.app.toml.example`
- `docs/DISCOVERY_RELEASE.md`
- `docs/APP_RELEASE.md`
- `docs/architecture/001-rust-v2-allsource-foundation.md`
