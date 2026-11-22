# Workspace Territory Map

Source item: affectUI Pro `components/repo-map` (functional reference only; see README).

## Purpose
Shows a codebase as a set of files grouped by folder, sized by line count, with the coding agents working in it placed on the files they are in. A person can see which files are being edited, which are only read, which files two agents share, and what each agent changed. It helps people avoid collisions between agents.

## Data
| Field | Type | Meaning |
|---|---|---|
| `file_list` | `RepoFile[]` | Each: `file_path`, `line_count`, `change_log?`. The folder part of the path groups the file. A path listed twice is shown once. |
| `change_log` | `{ agent_id, lines_added?, lines_removed? }[]` | Per file, oldest first. The most recent editor's agent mark sets the file's emphasis. |
| `agent_list` | `RepoAgent[]` | Each: `agent_id`, `agent_name`, `agent_emphasis?`, `current_file?` (current location), `agent_action?` (`"edit"` default or `"read"`), `agent_task?`. |
| `repo_name` | string | Repository name shown in the header. |
| `branch_name` | string, optional | Branch name shown in the header. |
| `folder_depth` | number | How many folder levels group files. Default 2. |
| `focused_agent_id` | string, optional | Controlled focus on one agent's files. |

Stable ids for agents and a stable file order keep the layout from shuffling as data arrives.

## Anatomy
In reading order:
1. Header: repository name, branch, and a count "N of M files changed".
2. Agent bar: one control per agent, showing its mark, name and the number of files it touched.
3. Map: a treemap. Each folder is a labelled group; each file is a tile whose area is proportional to its lines. Tiles show the file name and, when there is room, the count of changes.
4. Agent marks placed on the tiles where they currently are.
5. File card (on focus or pointer): full path, lines, the agents in it, and the change history with who changed what.

## States
- **Unchanged file:** neutral tile.
- **Edited file:** tile filled with its last editor's emphasis from the bottom up; once an agent moves on, the fill stays.
- **Read-only file (agent action read):** tile shows a dashed edge rather than a fill.
- **Conflict:** two agents editing the same file; the tile shows an alert treatment that pulses until one agent moves on.
- **Agent focused:** that agent's files stay clear and the others fade.
- **No agents:** the map shows change state only.
- **File card open:** the tile is highlighted and its card lists details.

Transitions: an agent moving to a new file moves its mark; starting an edit fills the new tile; leaving a file leaves its fill in place.

## Interactions
- **Pointer, tile:** shows the file card; tapping an agent in the bar focuses that agent.
- **Pointer, agent in the bar:** toggles focus on that agent.
- **Keyboard, Tab:** moves to the agent bar, then to the map as one stop.
- **Keyboard, arrow keys on the map:** walk files, folder by folder; each focused file shows its card.
- **Keyboard, Home and End:** first and last file.
- **Keyboard, Enter or Space on an agent:** toggles focus on that agent.
- **Keyboard, Escape:** hides the file card, or clears agent focus if no card is open.
- **Touch:** tap a tile for its card; tap an agent to focus it.

## Motion (functional)
- An agent's mark moves from one tile to the next with a soft transition, showing where it went.
- A tile fills from the bottom upward when an agent starts editing it; a sheen moves across while the edit continues.
- A conflict pulses until resolved.
- Reduced-motion preference: marks jump, tiles change at once, nothing pulses.

## Accessibility
- Each file tile's accessible name gives its path, its line count, the agents in it, and any conflict, e.g. "a path, 240 lines, conflict, two agents editing".
- Agent controls are toggle buttons that state the number of files each touched.
- A polite live region reports each time an agent changes to a different file.
- Focus rings are visible for keyboard users and are drawn inside tiles so the map does not clip them.

## Responsive
- The map keeps its height and re-lays out files for any width. File names and change counts appear while there is room.
- At narrow widths the map is taller, the change count moves to its own line, and agent controls wrap.
- Long paths and names truncate with an ellipsis; the full text is in the file card.

## Composition hints
- Card for the container.
- Treemap layout primitive.
- Avatar-style marks for agents.
- Toggle buttons for the agent bar.
- Popover or side card for file details.
- Badge for conflict and read state.
- Polite live region.

## Acceptance checks
1. Two files with 100 and 300 lines render with tile areas in a 1 to 3 ratio.
2. A file with two edits shows the emphasis of the most recent editor.
3. An agent whose action is read leaves its file with a dashed edge and no fill.
4. Two agents editing the same file produce a conflict state on that tile until one of them moves on.
5. Focusing one agent fades every file that agent is not in.
6. Keyboard arrows move focus between files in the same folder before moving to the next folder.
7. Pressing Escape with a file card open hides it.
8. The file card lists the full path, the line count, the agents present and the edit history.
9. Changing the order of input files does not reorder the existing tiles when the file set is unchanged.

## Amendment: structure and demo

**Archetype:** a treemap of one repository. Folders are labelled groups, files are tiles sized by line count, and agents are marks sitting on the files they are in. An agent bar sits above the map.

**Parts:**
- Header: repository name, branch name, and "n of m files changed".
- Agent bar: one toggle per agent with its mark, its name and the number of files it touched.
- Map: a labelled group per folder; a tile per file with its name and, where there is room, its change count; fill from the most recent editor; a dashed edge for a read-only file; a distinct conflict treatment for a file two agents are editing.
- Agent marks on the tiles.
- Status line below the map, announcing when an agent moves to another file.
- File card on focus or pointer: full path, line count, agents present, and edit history.

**Gallery demo:** repository `ledger-api`, branch `fix/rounding`. 14 files in 5 folders (api, billing, db, tests, docs). Five files changed. Three agents: "Billing agent" (2 files), "Ledger agent" (2 files), "Docs agent" (1 file). At rest: two edited tiles filled by different agents, one read-only tile, one conflict tile where two agents are on the same file. Status line: "Ledger agent is editing billing/rounding.ts". No file card open.

**Acceptance checks (continued):**
10. The gallery at rest reads "5 of 14 files changed", shows five labelled folder groups, and three agent toggles each with its file count.
11. At rest the map shows two fills from different agents, one dashed read-only tile and one conflict tile. No file card is open.
12. The status line names the agent and the file it is in, and changes when an agent moves.
13. The tiles fill the map's box with no unplaced area, and each tile's area stays proportional to its line count.

## Amendment 3: structure and demo

**Replaces** checks 10 and 11: the fourteen-file gallery and its fill layout at rest, and supersedes the earlier amendment's gallery demo. Checks 12 and 13 stand; the status line text changes as below.

**Archetype:** a treemap of a whole repository at its full file count. Folders are labelled groups, files are tiles, and agents sit as marks on the tiles they are in.

**Parts, top to bottom:**
- Header: repository name and branch at the left; "n of m files changed" at the right.
- Agent bar: one toggle per agent, with its mark, name, and the count of files it touched (edited or read).
- Map: one labelled group per folder and one tile per file. A tile shows the file name and line count. A changed file also shows its added and removed line counts. A read-only file carries a text mark. A file that two agents are editing carries a conflict text mark. Fill comes from the most recent editor.
- Agent marks on the tiles they occupy.
- Status line under the map, naming the agent and its file.
- File card: closed at rest.

**Gallery demo (our data):** repository `tally-service`, branch `fix/cent-rounding`. Twenty-three files in five folders: `api` (6 files), `payments` (5), `store` (4), `checks` (5), `notes` (3). Six changed files:
- `payments/rounding.ts`, 180 lines, +18 −6, edited by two agents: conflict.
- `payments/invoice.ts`, 240 lines, +12 −4, Payments agent.
- `store/cursor.rs`, 150 lines, +30 −2, Store agent.
- `store/retry.rs`, 110 lines, +9 −1, Store agent.
- `notes/rounding-note.md`, 90 lines, +14, Notes agent.
- `checks/rounding.test.ts`, 140 lines, +22 −3, Payments agent.

One read-only file: `api/quotes.ts`, 95 lines, read by Payments agent. Agent toggles: Payments agent touched 4 files, Store agent 3, Notes agent 1. Status line at rest: "Store agent is editing store/retry.rs".

**Acceptance checks (continued):**
14. At rest the header reads "6 of 23 files changed". The map shows five labelled folder groups holding all 23 tiles, with no unplaced area.
15. At rest the agent bar shows three toggles with their counts: Payments agent 4, Store agent 3, Notes agent 1.
16. At rest there is exactly one read-only tile (`api/quotes.ts`), with its text mark and no fill, and exactly one conflict tile (`payments/rounding.ts`). No file card is open.
17. Each changed tile shows its added and removed counts. Unchanged tiles show their line count alone.
18. The status line names the agent and the file it is in, and changes when that agent moves to another file.
