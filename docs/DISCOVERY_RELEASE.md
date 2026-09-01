# Public discovery release

This runbook turns `apps/web` into a crawlable, hardened Fly.io release. It
keeps product identity out of the starter: every bet supplies its own public
origins, name, summary, modification date, and runtime policy.

Search visibility is distribution evidence, not commercial evidence. Search
impressions, visits, AI citations, and clicks never satisfy a purchase gate.

## 1. Choose runtime policy

- `static`: no client event handlers. Staging removes Dioxus boot scripts,
  JavaScript, and WASM. Use for a public site made entirely of links, native
  HTML controls, and documents.
- `hydrate`: client event handlers exist. Staging moves generated inline
  bootstrap code into content-hashed same-origin assets so production CSP does
  not need `unsafe-inline`.

`cargo xtask stage-web` rejects `static` when rendered HTML contains a client
event handler. Do not work around this check; select `hydrate` or remove the
interaction.

## 2. Build verified SSG output

Set product values in the same process that compiles `apps/web`. These values
become canonical URLs, metadata, structured data, brand text, and app links.

```console
export PUBLIC_SITE_URL=https://your-product-domain.com
export PUBLIC_APP_URL=https://app.your-product-domain.com
export PUBLIC_PRODUCT_NAME='Product name'
export PUBLIC_PRODUCT_SUMMARY='One literal, evidence-safe explanation between 40 and 320 characters.'
export PUBLIC_SOCIAL_IMAGE_URL=https://your-product-domain.com/og.png

CARGO_INCREMENTAL=0 dx build --package web --platform web --release --ssg --fullstack true --force-sequential true --debug-symbols false
```

The exact command is verified against Dioxus CLI 0.7.10 in this workspace. It
renders every route returned by the `static_routes` server function into
`target/dx/web/release/web/public`.

## 3. Stage release boundary

```console
export PUBLIC_LAST_MODIFIED=YYYY-MM-DD
export PUBLIC_DISCOVERY_RUNTIME=static
# Use `hydrate` when any public route has client-side interactions.

cargo xtask stage-web
```

Output lands in ignored `.fly-artifacts/web`. Staging fails when compiled HTML:

- contains localhost or placeholder origins;
- lacks canonical product identity or JSON-LD;
- contains handlers under `static` policy; or
- retains inline executable scripts under `hydrate` policy.

It also writes `robots.txt`, `sitemap.xml`, `llms.txt`, and a stable IndexNow
verification key. `sitemap.xml` contains only routes present in SSG output.
Transitive asset pruning removes stale Dioxus hashes left by earlier builds;
only files reachable from current HTML, JavaScript, CSS, or WASM ship.

### Google Search Console HTML verification

When Search Console provides an HTML verification filename, rebuild staging
with the filename only:

```console
export GOOGLE_SITE_VERIFICATION_FILE=google0000000000000000.html
cargo xtask stage-web
```

Staging writes exact required body. Never commit verification tokens to this
starter.

## 4. Configure Fly.io

Copy `deploy/fly.web.toml.example` to product repository as `fly.web.toml`.
Replace app name. Keep existing product region when one exists; otherwise use
recommended reversible default `lhr` for a UK bet.

```console
fly deploy --config fly.web.toml --remote-only
```

Deployment serves staged files through pinned Nginx with:

- content-hashed assets cached for one year;
- HTML revalidated instead of cached forever;
- clean nested-route fallback, never a blanket SPA `200`;
- privacy-minimized access logs;
- CSP without inline scripts, framing disabled, and restrictive permissions;
  hydrated Dioxus retains required `unsafe-eval` and inline-style allowances,
  while `static` releases ship no JavaScript or WASM;
- one warm machine for stable first-byte latency.

Deploy only through existing Fly.io configuration after product repository has
its own app and domains.

## 5. Verify deployed surface

Run against public canonical domain, not `fly.dev` fallback:

```console
curl -fsSI "$PUBLIC_SITE_URL/"
curl -fsS "$PUBLIC_SITE_URL/robots.txt"
curl -fsS "$PUBLIC_SITE_URL/sitemap.xml"
curl -fsS "$PUBLIC_SITE_URL/llms.txt"
curl -fsS "$PUBLIC_SITE_URL/health"
```

Required checks:

- root, each sitemap route, discovery files, CSS, JS, and WASM return expected
  status and content type;
- canonical, Open Graph, description, WebSite, and WebPage metadata use public
  domain;
- Open Graph image returns `200`, uses a social-crawler-compatible raster
  format, and accurately previews this product;
- CTA reaches `PUBLIC_APP_URL` and every navigation target resolves;
- browser console has no CSP, WASM, hydration, or routing errors;
- keyboard traversal, visible focus, 200% zoom, 320 CSS-pixel reflow, reduced
  motion, labels, landmarks, and contrast meet WCAG 2.2 AA;
- mobile and desktop Lighthouse runs retain saved results; use PageSpeed
  Insights for public releases;
- Chrome UX Report data is recorded when available; “no field data” stays
  unknown, never converted into a pass.

## 6. Submit discovery endpoints

### Google Search Console

- Add domain property and complete DNS or HTML verification.
- Submit `$PUBLIC_SITE_URL/sitemap.xml`.
- Inspect root and one newly published route.
- Request indexing through Search Console UI when useful. Google Indexing API
  is not used for normal product pages.
- Record verification, sitemap status, inspected URL, and dates in bet evidence.

### IndexNow

After successful deployment:

```console
cargo xtask indexnow
```

Command first verifies deployed key file, then submits canonical root. HTTP 200
or 202 is success; every other status fails.

## 7. Capture evidence without moving gate

Record artifacts, not conclusions:

- deployed commit and Fly release;
- route and metadata checks;
- saved Lighthouse/PageSpeed results;
- Search Console verification and sitemap receipt;
- IndexNow response;
- CrUX availability or explicit absence;
- accessibility browser notes.

Do not promote from clicks, impressions, rankings, citations, praise, or this
technical readiness work. Original customer payment or usage gate remains
unchanged.
