# Application release

This runbook prevents a product app from shipping rust-v2 identity, localhost
API calls, or a logo link back to a development server.

## Build

Product identity is compile-time configuration for Dioxus WASM:

```console
export PUBLIC_SITE_URL=https://your-product-domain.com
export PUBLIC_API_URL=https://api.your-product-domain.com
export PUBLIC_PRODUCT_NAME='Product name'

CARGO_INCREMENTAL=0 dx build --package app --platform web --release --debug-symbols false
cargo xtask stage-app
```

`stage-app` copies output to ignored `.fly-artifacts/app`, externalizes Dioxus
bootstrap scripts, writes a blocking `robots.txt`, and fails when bundle lacks
compiled product/site/API identity or contains localhost placeholders. It also
removes stale hashed assets left by earlier Dioxus builds.

App brand always links to `PUBLIC_SITE_URL`. Dashboard remains a separate
internal navigation item. App HTML carries `noindex,nofollow,noarchive` because
authenticated product state is not a discovery surface.

## Deploy

Copy `deploy/fly.app.toml.example` to product repository as `fly.app.toml`, set
existing Fly app name and region, then deploy only through that configuration:

```console
fly deploy --config fly.app.toml --remote-only
```

The example keeps zero Machines warm while idle and autostarts a suspended
Machine on demand. App HTML and non-200 responses are `private, no-store`;
only content-hashed assets receive long public caching. Do not configure a
shared CDN rule for app routes. Measure first uncached navigation and the
commitment/fulfilment path before deciding whether a warm floor is needed.

## Verify

- `/`, `/login`, and each app route load without hydration, CSP, WASM, or
  routing errors.
- Brand/logo reaches canonical public site, never localhost or Fly fallback.
- Network requests target `PUBLIC_API_URL` and credential cookies behave on
  product domains.
- `robots.txt` disallows all crawling and HTML contains noindex policy.
- Keyboard navigation, focus visibility, form labels, errors, 200% zoom, 320
  CSS-pixel reflow, reduced motion, and contrast meet WCAG 2.2 AA.
- Missing paths follow product router behaviour; static host never disguises
  missing assets as HTML.
- App HTML, login routes, and missing responses are never shared-cached;
  fingerprinted assets may be immutable, and security headers remain present.
- A suspended-resume request reaches the product safely; a payment or form
  action is not treated as complete until authoritative receipt exists.

Record deployed commit, Fly release, route checks, browser console, and mobile
reflow in bet evidence. These are readiness artifacts, not promotion evidence.
