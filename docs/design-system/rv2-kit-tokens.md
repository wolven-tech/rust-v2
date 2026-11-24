# rv2-kit tokens

Every colour, shadow, radius, spacing step and type size in `crates/rv2-kit`
comes from a `--kit-*` custom property defined in
`crates/rv2-kit/assets/kit.css`. Component stylesheets name tokens by their job
and never contain a literal colour; `cargo xtask ci` enforces that (step
`no hardcoded hues`).

## How an app themes the kit

Most tokens read an rv2 register first and fall back to rv2's own value:

```css
--kit-surface: var(--color-surface, #ffffff);
--kit-id-1: var(--color-id-1, #2563eb);
```

An app sets the registers (`--color-*`) for each theme it ships, on `:root` or
on any element carrying `data-theme`. The kit follows without any kit-specific
CSS. Tokens without a register (tint recipe, shadows, radius, spacing, type)
are overridden directly by setting the `--kit-*` property on the same element.

Tokens are declared on `:root` and on every `[data-theme]` element, because a
custom property resolves its `var()` where it is declared: a token set only on
`:root` never sees a theme applied to a subtree.

Kit-owned dark values apply under `[data-theme="dark"]` and `.dark` only. The
kit does not read `prefers-color-scheme`: the app decides the theme and says so
on the element, so a stylesheet never disagrees with the app about which theme
is showing.

## Surfaces and ink

| Token | Register | Job |
|---|---|---|
| `--kit-ground` | `--color-ground` | Page or canvas behind cards |
| `--kit-surface` | `--color-surface` | Card and panel fill |
| `--kit-raised` | `--color-raised` | Chips, tracks, hover fills on a surface |
| `--kit-ink` | `--color-ink` | Primary text |
| `--kit-ink-hover` | `--color-ink-hover` | Text on hover |
| `--kit-ink-muted` | `--color-ink-muted` | Secondary text |
| `--kit-ink-faint` | `--kit-ink-muted` | Tertiary text, axis labels. Equal to muted so it passes AA on every fill; tertiary text differs by size, not colour. `--color-ink-faint` is not read |
| `--kit-on-ink` | `--color-on-ink` | Text on an ink or accent fill |
| `--kit-rule` | `--color-rule` | Borders, edges, connectors |
| `--kit-rule-soft` | `--color-rule-soft` | Dividers, card outlines |
| `--kit-accent` | `--color-accent` | Selection, focus, primary action |

## Status

| Token | Register | Job |
|---|---|---|
| `--kit-ok`, `--kit-ok-soft`, `--kit-ok-ink` | `--color-takeable*` | Done, healthy, allowed |
| `--kit-warn`, `--kit-warn-ink` | `--color-caution*` | Needs a person, waiting, near a limit |
| `--kit-fault`, `--kit-fault-ink` | `--color-fault*` | Failed, refused, over a limit |

Orange and red mean status and nothing else. A chart category, an agent or a
tool is never drawn in the warn or fault colours; use identity or series
tokens, which contain no orange or red.

Status words use one vocabulary in title case (Done, Running, Waiting, Needs
you, Failed, Paused, Skipped) and render as a pill with a dot.

## Actions

| Token | Default | Job |
|---|---|---|
| `--kit-primary`, `--kit-on-primary` | `--kit-accent`, `--kit-on-ink` | The one primary action per view: approve, allow, apply, deploy, run, send |
| `--kit-risky`, `--kit-on-risky` | `--kit-fault-ink`, `--kit-on-ink` | A deliberately risky action (grant broad access, delete). Not a second primary |

## Identity and series

| Token | Register | Job |
|---|---|---|
| `--kit-id-1` … `--kit-id-6` | `--color-id-1` … `-6` | Tells agents, tools and people apart. Chosen by index |
| `--kit-id-1-soft` … `--kit-id-6-soft` | derived | 14% of the identity colour over `--kit-surface` |
| `--kit-series-1` … `--kit-series-6` | `--color-series-1` … `-6` | Chart series. Never used for identity |

## Tinted marks

A component that gives each entity its own hue keeps only the hue as a local
variable and composes the rest from these:

```css
background: oklch(var(--kit-tint-bg-l) var(--kit-tint-bg-c) var(--kit-x-hue));
color: oklch(var(--kit-tint-ink-l) var(--kit-tint-ink-c) var(--kit-x-hue));
```

| Token | Light | Dark | Job |
|---|---|---|---|
| `--kit-tint-bg-l`, `--kit-tint-bg-c` | 95%, 0.035 | 30%, 0.05 | Mark fill |
| `--kit-tint-ink-l`, `--kit-tint-ink-c` | 38%, 0.1 | 88%, 0.08 | Text or glyph on the fill |
| `--kit-tint-strong-l`, `--kit-tint-strong-c` | 62%, 0.14 | 72%, 0.14 | Bars, rings and edges in the entity's hue |

## Gradients

| Token | Default | Job |
|---|---|---|
| `--kit-gradient-accent` | 135deg, `--kit-accent` to 45% `--kit-accent` over `--kit-id-1` | The accent gradient: a card face or the fill of gradient text. Reads the accent and identity registers, so light, dark and any app theme each get their own. Pair it with `--kit-on-ink` text only |

Gradient text takes a gradient name and reads `--kit-gradient-<name>`; an
undefined name falls back to a plain accent-to-identity gradient, never a status colour.

## Depth, overlay, special

| Token | Job |
|---|---|
| `--kit-shadow` | Raised element (button, node) |
| `--kit-shadow-card` | Resting card |
| `--kit-shadow-pop` | Popover, menu, tooltip |
| `--kit-scrim` | Backdrop behind a modal or drawer |
| `--kit-glass` | Translucent highlight laid over a fill |
| `--kit-think` | Reasoning or thinking spans, distinct from tool and wait |

## Shape, spacing, type, motion

| Token | Default | Job |
|---|---|---|
| `--kit-radius-sm` | 6px | Chips, small buttons |
| `--kit-radius` | 10px | Controls, nodes |
| `--kit-radius-card` | 12px | Cards and frames |
| `--kit-radius-lg` | 14px | Large panels, canvases |
| `--kit-radius-pill` | 999px | Pills, badges, chips and tracks with fully round ends. A square theme sets it to the control radius |
| `--kit-space-1` … `--kit-space-6` | 4, 8, 12, 16, 20, 24px | Padding and gaps. A compact or roomy theme overrides the scale |
| `--kit-gap` | 12px | Default gap between siblings |
| `--kit-text-xs` … `--kit-text-xl` | 11, 12, 13, 14, 16px | Type scale. Nothing in the kit is smaller than `xs` |
| `--kit-text-display` | `clamp(2rem, 4.2vw, 3.25rem)` | Display heading size, fluid from 32px to 52px. Headlines use it instead of their own clamp |
| `--kit-font`, `--kit-mono` | system stacks | Families |
| `--kit-ease`, `--kit-fast`, `--kit-slow` | 140ms, 320ms | Motion. Reduced motion sets both durations to 1ms |
