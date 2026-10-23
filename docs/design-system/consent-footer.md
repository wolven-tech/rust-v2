# Shared consent footer

`rv2-ui` owns the consent presentation used by product marketing sites. Its
browser renderer has no analytics, cookie or storage implementation. Product
adapters decide public-route eligibility, read saved consent, honor browser
privacy signals, and start or stop their configured analytics provider.

## Loading

Dioxus pages render `rv2_ui::ConsentFooter {}` once. This loads
`CONSENT_FOOTER_SCRIPT` and `CONSENT_FOOTER_STYLESHEET`. Static servers can serve
`CONSENT_FOOTER_JS` and `CONSENT_FOOTER_CSS` from the same module. Other renderers
use these exact assets rather than implementing a second panel.

An adapter can mount immediately when `window.Rv2ConsentFooter` exists, or listen
for `window`'s `rv2-consent-ready` event when scripts load asynchronously.

```javascript
const footer = window.Rv2ConsentFooter.mount({
  id: 'marketing-consent',
  footer: document.querySelector('footer'),
  choice: null, // 'accepted', 'declined', or null
  blocked: false,
  storageUnavailable: false,
  title: 'Analytics preferences',
  ariaLabel: 'Optional analytics',
  description: 'Optional analytics help us improve this website.',
  privacyHref: '/privacy',
  allowLabel: 'Allow analytics',
  declineLabel: 'Keep analytics off',
  withdrawLabel: 'Withdraw consent',
  onChoice(choice) { saveAndApplyConsent(choice); }
});
footer.update({ choice: 'declined', blocked: true });
// Call during route or component cleanup.
footer.destroy();
```

`footer` defaults to the first footer, then the body. The panel itself always
mounts under `body`: transforms and `content-visibility` on marketing sections
must not trap a fixed panel below the viewport. Its reopen button lives in the
footer. Replacing an instance with the same ID destroys its previous DOM.

Only unanswered consent initially expands. Accepting, declining or withdrawing
collapses the panel. Dismissing unanswered consent reports `declined`; dismissing
an existing choice preserves it. The footer's **Privacy preferences** control
reopens the panel and receives focus after dismissal. External choice updates
collapse an open panel without calling `onChoice`. Privacy signals disable
acceptance. Unavailable storage leaves explicit consent available for the current
visit and explains that the preference cannot persist. The adapter must enforce privacy signals in
its tracking code; presentation is not an authorization boundary.

## Product design tokens

Set these on `:root` or `body`, because the panel escapes footer ancestors:

| CSS variable | Purpose |
| --- | --- |
| `--rv2-consent-background`, `--rv2-consent-text` | Panel colors |
| `--rv2-consent-border` | Panel and button borders |
| `--rv2-consent-button-background`, `--rv2-consent-button-text` | Equal-weight choice buttons |
| `--rv2-consent-font`, `--rv2-consent-link` | Product typography and link color |
| `--rv2-consent-width`, `--rv2-consent-radius` | Content width and button corners |
| `--rv2-consent-focus`, `--rv2-consent-z-index` | Focus ring and overlay order |

The panel is capped at 65 dynamic viewport height with internal scrolling,
safe-area padding, mobile wrapping and no animation. Both choices remain
equally prominent. No preselected consent, timeout acceptance or scroll-based
acceptance is supported.

## Product checks

`cargo test -p rv2-ui` verifies that repeated Rust component mounts load exactly
one canonical script and stylesheet through the Dioxus document provider.

`crates/rv2-ui/tests/consent-footer.browser.cjs` drives the real assets at desktop
and mobile widths, including a transformed footer. It verifies viewport anchoring,
dismissal, focus return, reopening, acceptance, withdrawal, browser privacy,
unavailable storage, external updates, duplicate mounts and teardown.

Run with an installed Playwright module, without adding a package manager to
this Rust workspace:

```sh
PLAYWRIGHT_MODULE=/absolute/path/to/playwright node crates/rv2-ui/tests/consent-footer.browser.cjs
```

Product gates must additionally prove that no analytics request occurs before
acceptance, after withdrawal, on excluded routes or under browser privacy signals.
Those checks belong to the product adapter; the shared component emits no network
requests itself.
