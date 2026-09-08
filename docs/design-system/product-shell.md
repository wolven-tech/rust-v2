# Product-shell patterns

`rv2_ui::product_shell` gives each bet stable navigation and task orientation
without owning product identity or routing.

## Import styling

Both starter apps already import:

```css
@import "../../../crates/rv2-ui/assets/product-shell.css";
```

Override semantic hooks at product boundary. Keep component class names intact.

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
  --rv2-shell-width: 80rem;
}
```

## Header

Use stable route keys. Labels and URLs may change without breaking current-page
state. CTA remains a link because navigation must stay openable in new tab and
crawlable.

```rust
ProductHeader {
    brand: "Product name",
    brand_subtitle: "Local calculator",
    items: vec![
        ProductNavItem::new("home", "Home", "/"),
        ProductNavItem::new("guide", "Guide", "/guide"),
    ],
    current: "guide",
    action: ProductHeaderAction::new("Open app", "https://app.example.com"),
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
        ProductNavItem::new("measure", "01 Measure", "#measure"),
        ProductNavItem::new("review", "02 Review", "#review"),
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
