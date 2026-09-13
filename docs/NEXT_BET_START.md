# Next bet: advanced start, lean activation

Status: foundation guidance, not a placed bet or proof of demand. The product
contract stays in that bet's `docs/BET.md`; changing execution priorities go in
`docs/IMPACT_PLAN.md`. Each bet gets its own repository imported from a pinned
commit of this foundation. Existing products are not migrated by this guide.

## First decision: one complete value circuit

Name one customer in a triggered situation, payer, decision-maker, specific
outcome, exact price or fee trigger, and external promotion and kill gates.
Write the shortest truthful route:

`customer question → canonical answer → usable first value → commitment → fulfilment → authoritative proof`

Before coding, specify one synthetic **success trace** and one **failure trace**.
Each trace names its opaque correlation ID, expected first-value output,
commitment or authorised submission, authoritative receipt or local export,
fulfilment status, retry/idempotency behaviour, customer recovery, and support
route. Analytics may reference the opaque ID but cannot become receipt truth.
The failure trace must show that a success message cannot appear before the
trusted write or delivery succeeds. No personal, property, quote, or health
input belongs in analytics or the trace document.

For a free local-only bet, a verified on-device output can be the authoritative
result; do not activate an API or AllSource to manufacture a server receipt.
For a paid or shared-data bet, reconcile provider payment/submission state to
the delivered output and refund/failed-delivery state before promotion.

## Choose surfaces; keep unused ones parked

| Surface | Placement decision | Release evidence if active |
| --- | --- | --- |
| Public web | Static rendered answer and direct first-value link; canonical host, robots, sitemap, metadata, truthful schema, owner and support links | Production crawl, real links, GSC ownership/inspection, mobile/desktop PageSpeed, above-fold action |
| Product web | Deterministic local-first outcome where honest | Browser task completion, errors, receipt, keyboard and 320px reflow |
| API | Park unless trusted write, entitlement, sync, or another named server-only action needs it | Auth/validation, bounded retries, idempotency, rate limit, health, rollback, receipt trace |
| AllSource | Park unless shared durable data is essential; never introduce a second datastore by default | Tenant-scoped read-after-write, retention/deletion/recovery, failure proof |
| Native app | Build only for a distinct at-location, offline, camera, share, notification, or repeat-use job | Shared domain-rule parity, safe areas/system bars, offline recovery, physical-device smoke, store review and public URL |
| Generative AI | Park unless a bounded, grounded, human-reviewed step helps the fixed gate | Deterministic fallback, edit/accept evidence; no model deciding price, consent, eligibility, or legal/health truth |

Keep `apps/api`, `apps/web`, `apps/app`, shared Rust domain crates, `rv2-ai`,
and `tooling/xtask` boundaries from the foundation. Replace sample posts and
rust-v2 identity with the bet's smallest circuit. An existing API crate or
mobile-shaped component is not a reason to deploy either service. Native paid
digital access needs current store-policy review, purchase/restore/refund
parity, and no disguised web-checkout bypass.

## Low-cost production profile

The copied Fly examples now use `auto_stop_machines = "suspend"` and
`min_machines_running = 0` for static web and app hosts. This removes their
**guaranteed idle CPU/RAM floor**, not rootfs, volume, egress, provider, or
request-driven charges. The previous two 256 MB examples kept two Machines
warm; at the published September 2026 rate cited in the portfolio
cost analysis (kept in `founder-mode/docs/STATIC_HOSTING_COST_ANALYSIS.md`),
that implied roughly $4.42/month in guaranteed compute per new bet before API
or data cost. Recheck [Fly pricing](https://fly.io/docs/about/pricing/) when
placing the next bet; this figure is a dated comparison, not a quote.
Do not report $0 total hosting without invoice evidence.

Use guarded edge caching for public GET/HEAD pages and static assets. The
foundation serves successful public HTML with shared TTL, fingerprinted
assets as immutable, and unversioned assets with a short TTL. App HTML,
private/personalised routes, forms, payment, receipts, webhooks, API responses,
health, and errors must not enter a public cache. Configure CDN bypass by host,
path, cookies, and authorisation state; an origin cache header alone does not
prove a CDN hit. Purge on urgent price, legal, or factual corrections.

At placement, state dated provider price sources, maximum acceptable **cost per
delivered outcome** relative to price or fee, unknown volume assumptions, and
whether a warm API exception is justified by measured checkout, webhook, or
submission reliability. After release, measure edge hit/miss, cold and warm
first-value completion, origin wake-ups, provider use, refunds, and support
incidents. Lower spend never excuses failed fulfilment.

## Search, identity, and distribution before scaled content

Pick one demand-backed question from current evidence. Publish one useful
canonical answer with a crawlable `<a href>` directly to the functioning tool,
application, or purchase route. Show who it is for, literal output, limits,
price or fee, creator/owner, source dates, and one real or clearly fictional
worked example. No circular guide-to-guide route, token-swapped pSEO, invented
review, or generic AI copy. Keep private and checkout routes out of the index.

Prepare a flagship proof asset and one channel-native contribution, but block
paid growth until delivery, value proof, failed transactions, and production
blockers are resolved. Search impressions, AI citations, installs, and clicks
diagnose reach; only qualified completion/commitment counts toward the bet gate.

## Bespoke 3D art is mandatory, decorative load is not

Create one product-specific scene showing the mechanism: a meaningful object,
its input or state, and its transformed output. Retain editable scene or
generation source, original render, rights/provenance, and labelled fictional
versus factual evidence. Derive a responsive static hero, worked-example
illustration, icon, social preview, and store art when native app applies.
No generic floating cubes, fabricated interface, fake customer proof, or
synthetic defect photo presented as real. Use responsive dimensions, modern
compressed formats and static fallback; render no WebGL on first paint.

Verify primary action stays visible above fold on phone, text remains readable,
images have meaningful alt or decorative `alt=""`, focus and reduced-motion
behaviour work, and art does not regress LCP, INP, CLS, or task completion.
Treat lab measurements as lab evidence; missing field data stays unknown.

## Release proof and improvement loop

Create separate readiness, analytics-accuracy, value-delivery, marketing, and
impact records from the bet skill. Verify support inbox receipt/reply, owner
identity, privacy/deletion, security headers, domain/TLS, signed store builds
where applicable, and rollback. `cargo xtask ci` cannot prove a Dioxus view:
open every changed browser route and run the success/failure traces. A staged
mobile build is not a published store listing; record track, review state,
signed artifact, and public URL separately.

After launch, freeze the commercial gate. Improve one reversible bottleneck at
a time. Baseline source-to-gate counts, change one element, compare qualified
completion and cost per delivered value, then keep, revert, or replace. Append
decision and evidence; do not rewrite the original bet after seeing results.

## Autoresearch record: foundation shape

Method follows [autoresearch](https://github.com/karpathy/autoresearch): fixed
evaluator, one candidate change at a time, keep/discard log. This was a manual
design-and-config run; the local Rust autoresearch CLI does not yet run these
browser or product evaluators automatically. Scores below measure **explicit
starter safeguards**, not sales, traffic, or live production readiness.

Frozen cases: local one-off tool, at-location checklist, sensitive two-sided
marketplace, cross-platform paid tool, and search-led guide. Keep customer,
price, and promotion gate fixed in each case. Ten contract checks are frozen in
the bet skill's `next-bet-shape.md`. Two additional starter checks are:

11. Public/app examples impose no unjustified warm floor; successful public
    HTML can be edge-cached, while private/error responses cannot.
12. Success and failure traces name authoritative receipt/output, correlation,
    idempotency, recovery, and support before a success UI can claim delivery.

| Candidate | Single change | Structural checks | Hard failure | Decision |
| --- | --- | ---: | --- | --- |
| Existing contract v4 + old rust-v2 release examples | Baseline | 10/12 | Warm static/app floors and public HTML revalidation create avoidable origin cost | Replace release examples only |
| Edge-first release profile | Zero warm floor plus guarded origin cache policy | 11/12 | None in frozen cases; cold-start reliability still requires live proof | Keep |
| Receipt-trace gate | Require one synthetic success and failure trace at placement | 12/12 | None in frozen cases; traces are not customer evidence | Keep |

Hard stops for future runs: invented customer/value proof, public caching of
private data, mandatory unused API/native service, payment-policy bypass,
unreviewed regulated claim, or a cost saving that breaks the delivery trace.
Next actual bet must validate its contract, test a real buyer/user case, and
replace these structural scores with measured completion, value, cost, SEO
activation, device UX, and art performance. Unknown remains unknown.

Decision sources: [Google Search Central](https://developers.google.com/search/docs/fundamentals/seo-starter-guide),
[Core Web Vitals](https://web.dev/articles/defining-core-web-vitals-thresholds),
[WCAG 2.2](https://www.w3.org/TR/wcag/),
[Fly autostop/autostart](https://fly.io/docs/launch/autostop-autostart/),
[Cloudflare cache rules](https://developers.cloudflare.com/cache/how-to/cache-rules/),
[Google Play payments](https://support.google.com/googleplay/android-developer/answer/9858738),
and [Apple review guidelines](https://developer.apple.com/app-store/review/guidelines/).
