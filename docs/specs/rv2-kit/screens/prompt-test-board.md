# Prompt Test Board Screen

Source item: affectUI Pro `screens/evals` (functional reference only; see README).

## Purpose
A team that writes instructions for an assistant uses this board to check whether each new version of a prompt passes a fixed set of test cases, to review proposed edits to the prompt one at a time, and to re-run the tests after a change.

## Data
| Field | Type | Meaning |
|---|---|---|
| workspace_name | string | Sidebar label. |
| account | { name; avatar_url? } | Account block. |
| prompt_name | string | Name of the prompt under test. |
| versions | { id; label: string; created_at: string; pass_rate: number }[] | Version history; the latest is current. |
| current_version | string | Id of the version displayed. |
| model | string | Model the tests ran against, shown as text. |
| last_run_text | string | For example "last run two hours ago". |
| key_figures | { label; value; delta; good_when: "up" \| "down" }[] | Pass rate, average score, median latency, and cost per 1,000 runs, each with its change from the previous version. |
| pass_rate_series | { version_label; pass_rate: number }[] | Bars or points per version (0 to 100). |
| proposed_changes | { id; kind: "add" \| "remove" \| "replace"; before: string; after: string; author: string }[] | Edits to the prompt waiting for review. |
| test_cases | { id; input: string; expected: string; output?: string; score?: number; max_score: number; latency?: number; status: "pass" \| "partial" \| "fail" \| "pending" }[] | Fixed cases with their latest result. |
| filter | "all" \| "pass" \| "partial" \| "fail" | Active filter on the test case list. |
| is_loading | boolean | Placeholders in every data region; header and filters stay. |
| is_empty | boolean | A prompt with no test cases yet: the test case region shows an empty state; charts and changes are hidden. |
| on_run_eval | () => void | Run evaluation button. |
| on_select_version | (id) => void | A version was selected. |
| on_accept_change / on_reject_change | (id) => void | Per proposed change. |
| on_accept_all / on_reject_all | () => void | Bulk decisions. |
| on_add_case | () => void | The empty state's add button and the list's add control. |
| on_open_case | (id) => void | A test case was opened. |

## Anatomy
Reading order:
1. Sidebar, left: workspace, navigation (Chat, Evals with a count, Datasets, Analytics, Prompts), the prompt list, account.
2. Header: the prompt name with a version menu, a current-version tag, the model name, the last run text, and the Run evaluation button.
3. Key figure row: four figure cards, each with label, value, and change from the previous version.
4. Pass rate by version: a bar or point chart across versions, with the current one emphasised, and a caption naming the number of versions and test cases.
5. Changes panel (right column on wide layouts): the current prompt text with proposed changes marked inline; a count; "Reject all" and "Accept all".
6. Test cases panel (below on narrow layouts): filter tabs with counts (All, Pass, Partial, Fail), then rows showing input, expected behaviour, output, score, latency, and status.

## States
- **Loaded.** The default.
- **Loading.** Placeholder shapes stand in for figures, bars, change lines, and test rows. The header, filters, and Run evaluation remain in place.
- **Empty.** The version stays at its first label; figures, chart, and changes are hidden; the test case region shows "No test cases yet" with an add button.
- **Running.** After Run evaluation, rows show pending status in turn; the button becomes a progress control; figures update when the run completes.
- **Change pending.** A proposed change is shown as an inline struck-through part and an added part. It stays pending until accepted or rejected.
- **Change accepted or rejected.** Accepted changes merge into the text; rejected changes leave the view. Both are removed from the pending count.
- **Filter active.** Only rows with the chosen status show; the tab shows its count.
- **Row selected or open.** A test case opens to show its full input, expected behaviour, output, and score rationale.
- **Version menu open.** Lists the versions; picking one switches the figures and highlights its bar in the chart.

## Interactions
- Run evaluation: click or Enter starts a run of all test cases.
- Version menu: Enter or Space opens it; arrows choose; Enter confirms; Escape closes it.
- Changes panel: Tab moves between changes; Y accepts the focused change; N rejects it.
- Accept all and Reject all: buttons with a count in their label.
- Test case rows: focusable; Enter or Space opens the row.
- Filter tabs: arrow keys move; Enter selects.
- Escape: closes an open row, then a menu.

## Motion (functional)
- Score bars and points fill in as results arrive.
- Accepting a change merges its text into the prompt while the surrounding text stays still.
- Rows move into their status group as their result comes in, without a full re-sort.

## Accessibility
- Filter tabs form a tab list with counts in their names.
- Each change states its kind and position in its label; Y and N are announced with the result.
- Each test row takes keyboard focus and opens on Enter or Space.
- Status is written as text next to its visual cue.
- Figures are labelled groups; the chart has a text summary listing each version's pass rate.
- Run progress is announced in a polite region at start and end.

## Responsive
- When there is not room for two columns, the changes panel moves beneath the test case region.
- On narrow screens, the test case table scrolls sideways inside its own box, with the input column kept first.
- The page fills its container.

## Composition hints
- Workspace frame (see Workspace Frame (Inset)).
- Stat card for each key figure with a change line.
- Bar chart by version with tooltip.
- Diff view with inline removals and additions, and per-change accept or reject.
- Tab strip with counts for filters.
- Data table with expandable rows and status badges.
- Select or menu for the version switch; buttons for run and bulk decisions.
- Empty state; skeleton placeholders.

## Acceptance checks
1. With `is_loading` set, figures, bars, and test rows show placeholders while the header and Run evaluation remain usable.
2. With `is_empty` set, the test case region shows its empty state, and its add button calls `on_add_case`.
3. Pressing Y on a focused proposed change calls `on_accept_change` with its id and removes it from the pending count.
4. Pressing N on a focused proposed change calls `on_reject_change` with its id.
5. Choosing the "Fail" filter shows only failing test rows, and the tab shows the matching count.
6. Pressing Enter on a test row opens its full details, including its output and score.
7. A key figure whose change matches `good_when` renders the positive tone, otherwise the negative tone, always with a sign word.
8. Selecting an older version in the version menu updates the figures and highlights its bar in the chart.
9. Run evaluation changes every row to pending and announces the start in a polite region.
10. At narrow width the page has no horizontal scroll; only the test table scrolls inside its box.
