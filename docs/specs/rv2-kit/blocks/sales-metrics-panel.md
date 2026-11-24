# Sales Metrics Panel

Source item: affectUI Pro `blocks/metrics-dashboard` (functional reference only; see README).

## Purpose
Shows a sales overview for a chosen period: headline metrics, one trend chart whose metric and style the person can change, and ranked breakdowns of top products and channels.

## Data
- `title`, `subtitle`.
- `metric_cards`: list of { id, label, value (formatted), delta_percent, direction: "up" | "down", series: number[] }. One card is the selected metric.
- `periods`: list of { id, label }, e.g. a week, a month, a quarter. `selected_period`, `on_period_change(id)`.
- `trend_series`: list of { date, value } for the chosen metric, with an optional previous-period series.
- `chart_style`: "area" | "bars"; `on_chart_style_change(style)`.
- `compare_enabled`, `on_compare_change(bool)`.
- `top_products`: list of { id, name, value }; `channels`: list of { id, name, value }.
- `on_export(format)`.
- `format_amount(value)`.

## Anatomy
In reading order:
1. Header: title, subtitle with the date range, period picker, export control.
2. Metric row: one card per metric. The selected card is marked and the others are de-emphasised.
3. Trend panel: the chart name and its metric label, a chart-style switch, a compare toggle, axis labels and the chart area.
4. Breakdown row: a top products list and a channels list, each with name and value rows scaled to the largest value.

## States
- Default: first metric selected, area style, compare off.
- Metric selected: that card is marked; the trend chart redraws for that metric; the other cards show de-emphasised values.
- Bars style: the trend area becomes one bar per point; hover shows the point's value.
- Compare on: a second series for the previous period appears with a legend entry.
- Loading: cards and chart show placeholders; breakdown rows show placeholder bars.
- Empty trend: the chart area shows a message and axes without points.
- Export in progress: the export control shows progress and is disabled.

## Interactions
- Click a metric card: selects it.
- Click the style switch: toggles area and bars; keyboard arrows move between the two options.
- Hover on the chart: shows a tooltip with the date, value and, when compare is on, the previous value; a marker tracks the nearest point.
- Compare toggle: switch on and off.
- Period picker: listbox; Enter picks, Escape closes.
- Export: opens a menu of formats; choosing one calls `on_export`.
- Keyboard: Tab order is period picker, export, metric cards, style switch, compare toggle, chart, breakdown rows. Left and Right on the chart step through points.

## Motion (functional)
- Changing the metric rolls the card values to their new numbers and redraws the chart from the baseline.
- Switching style morphs the chart; points become bars or the area takes the place of the bars.
- The compare series fades in after the main series has drawn.
- Breakdown rows grow to their scaled width once their values are known.
- With reduced motion preferred, changes are instant.

## Accessibility
- Metric cards are toggle buttons with a pressed state.
- The chart is a labelled group; its summary (metric, period, total) is read first, and point values are reachable by keyboard.
- The style switch is a radio group with names "Area" and "Bars".
- Delta values carry an up or down word, not only an arrow.
- Breakdown lists are tables with name and value columns.

## Responsive
- Wide: four metric cards in one row; trend panel full width; breakdown lists side by side.
- Medium: metric cards in two rows; breakdown lists stacked.
- Phone width: metric cards in a two-column grid; chart keeps full width with a shorter height; the style switch and compare toggle wrap onto a second line.

## Composition hints
- Card (metric cards and panels), toggle button, chart primitive (area, bar, single or two series), radio group, switch, listbox (period), menu (export), table (breakdowns), badge (delta), skeleton.

## Acceptance checks
1. Selecting a metric card marks it, mutes the others, and redraws the trend for that metric.
2. Switching the style to bars shows one bar per trend point and the tooltip reports the same value as the area style.
3. Turning compare on adds a second series and tooltip values include the previous period.
4. Hovering the chart moves a marker to the nearest point and shows its date and value.
5. Breakdown rows are sorted by value, largest first, and their widths scale to the largest row.
6. Export lists formats and calling one fires `on_export` with that format.
7. At phone width the metric cards form two columns and nothing scrolls sideways.

## Amendment 5: structure and demo

Archetype: dashboard block: header, a four-card metric row, one trend panel, and two ranked lists.

Visible at rest:
- Header: title, subtitle with place and date range, period segmented control (Week, Month, Quarter), export control at the right.
- Metric row: four cards in one row at wide widths, in this order: Revenue (marked as selected), Orders, Average order, Refund rate. Each shows a label, a value and a delta with a direction word.
- Trend panel: title "Revenue", a line and bars switch, a compare switch set to off, three value-axis labels, three date labels, and the chart area.
- Breakdown row: "Top products" and "Channels" lists, five rows each, with a column header "Revenue" at the right.

Gallery demo:
- Title "Sales", subtitle "Kiosk network · Last 30 days".
- Cards: Revenue $48.2k, up 12.4%. Orders 612, down 3.1%. Average order $78.80, up 2.2%. Refund rate 1.8%, down 0.3 pts.
- Trend: 21 daily points from Nov 1 to Nov 21, in dollars. Two local peaks and one dip. The 21 values add up to the Revenue card's value.
- Top products: Trail jacket $12.1k, Rain shell $9.4k, Fleece vest $8.2k, Wool socks $4.3k, Canvas cap $2.9k.
- Channels: Web shop $21.7k, Partner stores $11.4k, Marketplace $8.9k, Phone orders $3.6k, Events $2.6k.

Motion, settled frame: Revenue selected; trend drawn; breakdown rows at final width.
Moves: changing the metric rolls the card values and redraws the trend from the baseline. The style switch morphs points into bars and back. The compare series fades in after the main series. Breakdown rows grow to their widths.
Reduced motion: changes are instant.

Acceptance checks (continued):
8. At wide widths the four metric cards sit in one row in the order Revenue, Orders, Average order, Refund rate.
9. The top value label on the trend's value axis is at or above the highest daily value, and its unit is dollars in thousands, as on the Revenue card.
10. The trend has 21 points, at least two local peaks and one dip; it is not a straight line.
11. The 21 daily values add up to the Revenue card's value, within rounding.
12. Each breakdown list shows five rows, largest first, with bar widths scaled to the largest row.
