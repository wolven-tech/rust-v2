# Revenue Overview Screen

Source item: affectUI Pro `screens/analytics` (functional reference only; see README).

## Purpose
A business owner opens this page to see, in one pass, how much money came in,
which channels and products produced it, and whether the current period is
ahead of or behind the one before.

## Data
| Field | Type | Meaning |
|---|---|---|
| workspace_name | string | Shown in the sidebar header. Default "Juniper Works". |
| account | { name: string; avatar_url?: string } | Account block at the sidebar foot. Without an image, initials. |
| period_label | string | Text of the period selector, e.g. "Last 7 days". |
| is_loading | boolean | Replaces every data region with placeholders. |
| is_empty | boolean | No data yet: replaces data regions with one empty state. |
| on_connect | () => void | Primary action of the empty state. |
| key_figures | { item_name; item_value; change_text; change_sign: "up" \| "down" \| "flat"; good_when: "up" \| "down"; trend_bars: number[] }[] | Headline cards, shown in the given order. |
| channel_series | { name: string; daily: number[] }[] | Lines of the main chart, one value per day on a shared day axis. |
| breakdown_rows | { item_name; item_value; trend_points: number[]; trend_direction: "up" \| "down" \| "flat" }[] | Ledger-style rows: components of the total. |
| ranked_items | { item_name; item_value; share_pct: number }[] | Best sellers. `share_pct` is 0 to 100, relative to the first row. |
| sessions_series | { item_name: "This period" \| "Previous period"; daily: number[] }[] | Two-line comparison chart. |
| average_value | { current: string; previous: string; daily: number[] } | Single metric with a comparison line. |
| on_navigate | (label: string) => void | Sidebar item clicked. |
| on_export | () => void | Export action in the header. |
| on_save_report | () => void | Save-as-report action in the header. |

## Anatomy
Reading order, top to bottom:
1. Sidebar (collapsible, see Workspace Frame): workspace name, menu items, account.
2. Page header: page title, period selector, then the actions Export and Save as report.
3. Key figure row: one card per entry in `key_figures`. Each card holds its label,
   the value in the largest type on the card, a change line with a sign word and a
   small bar series.
4. Main chart card: heading, total for the period, change against the previous
   period, the multi-line chart, and a legend that names each series.
5. Breakdown card: rows of label, value and a tiny trend line per row.
6. Ranked items card: rows of label, value and a share bar.
7. Sessions card: two-line comparison chart with both totals.
8. Average card: current value, previous value, comparison line.

The empty and loading variants keep the header (title, period selector, actions)
and nothing else from items 3 to 8.

## States
- **Loaded.** All regions show their data. Default.
- **Loading.** Every card keeps its frame and title, and its content is secondary
  placeholders in the shape of the real rows, bars and lines. Header and sidebar
  stay interactive. Triggered by `is_loading`.
- **Empty.** Key figures, charts and lists give way to one empty state: a title
  "No revenue yet", one line on what will appear here, and the connect action.
  The header stays. Triggered by `is_empty`.
- **Change tone.** A change is positive when its `change_sign` equals `good_when`,
  negative when it does not, and neutral when `change_sign` is "flat". The sign word
  is always printed, so a visual cue is never the only signal.
- **Hovered chart day.** A marker and tooltip show the day and each series value
  for that day. Ends when the pointer or focus leaves the chart.
- **Chart focused.** A visible focus ring and a day cursor. Ends on blur.
- **Menu open** (period selector). Ends on selection, outside click or Escape.

## Interactions
- Pointer over the main chart: vertical guide follows the pointer, tooltip lists
  every series for the nearest day.
- Touch: press and drag across the chart moves the tooltip; lifting the finger
  keeps the last day shown until the next touch.
- Keyboard focus order: sidebar items, period selector, Export, Save as report,
  key figure cards (not interactive, skipped), main chart, breakdown rows, ranked
  rows, sessions chart, average chart.
- Chart with keyboard focus: left and right arrows move the day cursor by one day;
  Home and End jump to first and last day; Escape removes the cursor.
- Period selector: Enter or Space opens; arrows choose; Enter confirms; Escape
  closes and returns focus to the selector.
- Export and Save as report: click or Enter calls the callback.
- Sidebar item: click or Enter calls `on_navigate` with its label and marks it active.

## Motion (functional)
- Figures change value in place with a rolling-digit effect, so a change reads as
  a delta rather than a jump.
- Bars and lines draw in once when the data first appears; they redraw only when
  the period changes, not on hover.
- Loading placeholders share one light sweep.

## Accessibility
- Each key figure is a labelled group whose label, value and change are read as one
  unit.
- Sidebar items, the period selector and header actions are buttons with visible
  labels.
- Each chart has a text summary (series name, total, change) and is reachable by
  keyboard; the day cursor announces the day and each series value.
- Decorative icons and spark bars are hidden from assistive technology.
- A period change is announced in a polite live region.
- Under reduced motion, rolling digits and draw-in are off; values appear at once.

## Responsive
- Phone width: the sidebar becomes a drawer opened from the header; key figures
  stack into two columns; charts stay full width; breakdown_rows, ranked and average
  cards stack below the charts.
- The ranked and breakdown lists never scroll sideways. Only a wide sessions chart
  may scroll inside its own box.
- The screen fills its container; the host supplies the height.

## Composition hints
- Workspace frame (sidebar and page area).
- Stat card with change line and mini bars, for key figures.
- Multi-series line chart with hover tooltip and keyboard day cursor.
- Ledger list (label, item_value, sparkline) for the breakdown.
- Ranked list with share bars.
- Select or menu for the period, button for header actions, badge for change sign.
- Empty state, skeleton placeholders and a loading wrapper.

## Acceptance checks
1. With `is_loading` set, every card shows placeholders, its title stays, and header buttons still respond.
2. With `is_empty` set, no figure, chart or list is visible; the connect control calls `on_connect` once.
3. A key figure with `change_sign` "up" and `good_when` "up" renders the positive tone; with `good_when` "down" it renders the negative tone.
4. Hovering a point in the main chart shows one tooltip row per series for that day.
5. With the chart focused, pressing right moves the day cursor forward one day and announces it.
6. Clicking a sidebar item calls `on_navigate` with that item's label and shows it as active.
7. A ranked row whose `share_pct` is 50 draws its share bar at half the width of the full track.
8. At phone width the page has no horizontal scroll; the sidebar is reachable through its drawer.
9. Choosing a new period in the selector updates the period label and fires a live announcement.
10. With reduced motion requested, values render without rolling and bars without draw-in.
