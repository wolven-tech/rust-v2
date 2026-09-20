# Ledger

The dark identity for `apps/hiring`. A warm, restrained reading space where
negative results carry no apology, and the rare match earns its shine in gold.

Confirmed on screen 2026-09-20.

## What the palette is for

The register's job is to say no. Of 30 live listings on the UK's specialist
outside-IR35 contract board, six are remote enough, one carries a mandate to
build a department, and none are both. A reader spends an hour at a time on a
screen of disqualifications.

So the palette has two jobs, and they pull against each other: make a long list
of failures readable without fatigue, and make the handful that pass impossible
to miss. Everything below follows from that.

## Boldness is spent once

A role clearing both essential filters casts the light system's hard zero-blur
offset shadow, in gold:

```css
box-shadow: 0.5rem 0.5rem 0 #f4c350;
```

**Nothing else on the page casts a shadow at all.** Six of these across 103
companies. The restraint is what makes them work — the gesture means "this one
is possible" and it cannot mean that if a card carries it too.

Under `prefers-reduced-motion` it collapses to an outline in the same gold. The
meaning survives; the displacement goes.

## Tokens

Every colour is named for its job, never its hue. `apps/hiring/tests/tokens.rs`
fails the build if a component writes `text-slate-600` again — a hardcoded hue
silently opts that element out of every theme, and the failure stays invisible
until one paragraph is light grey on a dark page.

| Token | Value | Why that value |
|---|---|---|
| `--color-ground` | `#1a1714` | Warm charcoal. `#0B0B0B` and `#111` are the tell of every generated dark theme; this keeps the paper theme's thermal character so the two read as one hand |
| `--color-surface` | `#2d2622` | 1.08:1 on ground — a card, not a step |
| `--color-raised` | `#34302b` | Hover only |
| `--color-ink` | `#e8e5e2` | 15.8:1. Warm cream rather than white, for the reason the paper theme is `#fbf7ed` rather than white |
| `--color-ink-muted` | `#c8bfb6` | **7.5:1, not the 4.5:1 AA allows.** Set by the 11px evidence quotes, not by the standard |
| `--color-ink-faint` | `#9a9086` | Untracked values and disabled controls |
| `--color-rule` | `#4a4340` | Table borders |
| `--color-rule-soft` | `#3a3430` | Barely visible structural dividers |
| `--color-accent` | `#a8c7fa` | Focus and controls. Deliberately lighter than the FTSE 100 blue so a focus ring never reads as market data |
| `--color-takeable` | `#42d962` | The only green on the page |
| `--color-fault` | `#ff7159` | A broken careers link |
| `--color-caution` | `#e0a84a` | A site that refused a scripted request, which is not the same as gone |

### Market identity

A reader scans a column by colour without reading a word, so these are signal.
Five hues kept apart by luminance as well as hue, so the table survives a reader
who sees none of them.

| Market | Colour |
|---|---|
| FTSE 100 | `#7ab0f0` |
| FTSE 250 | `#4fc9c9` |
| FTSE SmallCap | `#9d8df0` |
| AIM | `#dab644` |
| Targets outside UK indices | `#d98cc4` |

**No green here, and that constraint shaped the whole set.** Green means
takeable. Six roles out of 103 companies only read as rare if nothing else on
the page is green.

## How it was chosen, and what was overruled

Five independent directions were produced and judged on three lenses —
distinctiveness, legibility under density, and whether each read as a credible
dark counterpart to the light system. Ledger won at 7.2 of 10.

Two things in the winning direction were overruled:

- **It assigned FTSE 250 `#6dd96d`**, a bright green, while also declaring green
  reserved for takeable verdicts. Moved to teal, and SmallCap to indigo to keep
  five hues apart.
- **The Rust signal lost its hue entirely.** Five markets plus three verdicts
  plus gold is already at the limit of what a reader decodes. A tenth hue stops
  encoding and starts decorating, so a Rust role is a filled chip now and a
  mention is a quiet one.

A judge flagged the winner for trusting its own arithmetic without checking
11px under fatigue. That is why `--color-ink-muted` sits where it does.

The direction ranked third was the obvious one: invert the light system, make
its ink `#15231f` the ground. It scored 6.3 and a judge marked it
overconfident. Worth recording, because it is the answer anyone would reach for
first, including the person who set the brief.

## Refused

- **A tenth hue.** One colour per meaning.
- **Animated filter transitions.** The product's claim is honest judgement, and
  honest means immediate. The reader does not wait for a fade to learn whether
  they were right.
- **Soft drop shadows.** All shadows here are hard offset or none. A soft shadow
  disappears on a dark ground and would dilute the one gesture that carries
  meaning.
- **Pure white or pure black.** Both ends are offset to keep the light system's
  warmth and reduce strain over a long session.

## Changing it

The palette is one `@theme` block in `apps/hiring/assets/input.css`. Nothing
else needs touching, which is the point of the token layer. Run
`cargo xtask styles` after, and commit the generated stylesheet — the gate
checks it is current.

`rv2-ui` components resolve against the same token names, so any app consuming
the kit must declare them. `apps/app` and `apps/web` carry the subset the kit
touches; adding a kit component that uses a new token means adding it there too,
or that element renders unstyled in those apps only.
