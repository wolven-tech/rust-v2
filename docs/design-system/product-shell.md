# Product-shell patterns

`rv2_ui::product_shell` gives each bet stable navigation and task orientation
without owning product identity or routing.

## Import styling

Both starter apps already import:

```css
@import "../../../crates/rv2-ui/assets/product-shell.css";
```

Override semantic hooks at product boundary. Keep component class names intact.

Hooks that have a kit register default to it — `paper` to `--color-surface`,
`ink`, `muted` to `--color-ink-muted`, `rule`, `focus` to `--color-accent`, and
`done`/`done-ink` (completed process steps) to `--color-takeable`/`-ink` — so a
product that sets its `@theme` gets a header in the same palette with no
overrides here (UI3 in `../architecture/003-ui-semantic-isa.md`). `accent`,
`accent-soft` and `highlight` are the shell's own decoration and keep their
hex defaults. Override any of them only where the shell should differ from the
rest of the product.

```css
.my-product {
  --rv2-shell-paper: #fbf7ed;
  --rv2-shell-ink: #15231f;
  --rv2-shell-muted: #53665e;
  --rv2-shell-rule: #a8b5af;
  --rv2-shell-accent: #2456d8;
  --rv2-shell-accent-soft: #e7edff;
  --rv2-shell-highlight: #f4c350;
  --rv2-shell-focus: #183fb2;
  --rv2-shell-done: #087f5b;
  --rv2-shell-done-ink: #07684b;
  --rv2-shell-width: 80rem;
}
```

## Header

Every link is a `NavLink`. A route takes a stable key through
`NavLink::route`; labels and URLs may change without breaking current-page
state. A call-to-action, footer link or in-page index entry is `NavLink::new`
and has no key, so it can never be marked current. `.external()` opens either
kind in a new tab. CTA remains a link because navigation must stay openable in new tab and
crawlable.

```rust
ProductHeader {
    brand: "Product name",
    brand_subtitle: "Local calculator",
    items: vec![
        NavLink::route("home", "Home", "/"),
        NavLink::route("guide", "Guide", "/guide"),
    ],
    current: "guide",
    action: NavLink::new("Open app", "https://app.example.com"),
}
```

Caller may pass `brand_mark` and `tail`. On narrow screens, route list and CTA
move into native `<details>` disclosure. Tail is secondary and hidden to avoid
header overflow.

## Navigation rail

Use `NavigationRail` for guide contents or policy siblings. Omit `current` for
in-page indexes; set it only when one item represents current page.

```rust
NavigationRail {
    label: "Guide contents",
    items: vec![
        NavLink::new("01 Measure", "#measure"),
        NavLink::new("02 Review", "#review"),
    ],
}
```

## Process rail

Use `ProcessRail` for three or more ordered stages. `active_step` is one-based.
Omit it for static process explanation. Use linked steps when sections exist.

```rust
ProcessRail {
    label: "Calculator progress",
    active_step: 2,
    steps: vec![
        ProcessStep::linked("Input", "Name job", "#input"),
        ProcessStep::linked("Work", "Enter facts", "#work"),
        ProcessStep::linked("Proof", "Review output", "#proof"),
    ],
}
```

## Motion and accessibility

`PageEntrance` runs once with opacity and transform. Reduced-motion preference
removes animation. Header, rail links, CTA, and disclosure keep 44px minimum
targets and visible focus. Components do not certify page-level WCAG AA;
products still verify contrast overrides, zoom, reflow, headings, errors, and
task completion in rendered browsers.
