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
