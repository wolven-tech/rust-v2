# Meta User Guide

Detailed guide for daily use of Meta.

[![asciicast](https://asciinema.org/a/N9NaULQ470ih2SV6aymeENmLq.svg)](https://asciinema.org/a/N9NaULQ470ih2SV6aymeENmLq)

## Table of Contents

- [Daily Workflow](#daily-workflow)
- [Viewing Logs](#viewing-logs)
- [Multi-Workspace Support](#multi-workspace-support)
- [Claude Code Integration](#claude-code-integration)
- [Tmux Navigation](#tmux-navigation)
- [Recording Demos](#recording-demos)
- [Troubleshooting](#troubleshooting)

## Daily Workflow

### Starting Your Dev Session

```bash
# Check everything is OK
meta doctor

# Start all services
meta dev

# Or start specific projects only
meta dev -p api web
```

### Navigation Guide

Meta displays this when starting:

```
╭─────────────────────────────────────────────────────────╮
│ Tmux Navigation Guide                                   │
├─────────────────────────────────────────────────────────┤
│ Navigate Panes:  Ctrl+B then Arrow Keys (← → ↑ ↓)      │
│ Zoom Pane:       Ctrl+B then Z (toggle full screen)    │
│ Show Numbers:    Ctrl+B then Q (then press number)     │
│                                                         │
│ Detach Session:  Ctrl+B then D (keeps running)         │
│ Stop Process:    Ctrl+C (in current pane)              │
│ Close Pane:      Ctrl+B then X (confirm with y)        │
╰─────────────────────────────────────────────────────────╯
```

### Working with Bacon

Each Rust project runs bacon with full interactive TUI:

- **t** - Switch to test mode
- **c** - Switch to clippy mode
- **r** - Switch to run mode
- **w** - Switch to watch mode

### Detaching and Reattaching

**Detach** (keeps everything running):
```
Press: Ctrl+B then D
```

**Reattach later:**
```bash
meta sessions          # List all sessions
tmux attach -t meta-<workspace>
```

**Stop everything:**
```bash
meta dev:stop
```

## Viewing Logs

Meta captures stdout/stderr from all dev processes.

### Log Locations

```
.meta/logs/
├── dev.log        # Lifecycle events (START/EXIT/RESTART)
├── api.log        # API project output
├── web.log        # Web project output
└── api.log.1      # Rotated backup (>10MB)
```

### Commands

```bash
meta logs              # List available logs
meta logs api          # View last 50 lines
meta logs api -l 100   # View last 100 lines
meta logs api -f       # Follow in real-time
meta status            # Show processes + recent events
```

## Multi-Workspace Support

Each workspace gets a unique tmux session based on directory name.

### Managing Multiple Workspaces

```bash
# List all active sessions
meta sessions

# Output:
# meta-backend (this workspace)
#   Panes: 3
# meta-frontend
#   Panes: 2

# Switch to another workspace
tmux attach -t meta-frontend

# Stop specific workspace
tmux kill-session -t meta-backend
```

## Claude Code Integration

Meta includes a skill for Claude Code AI agents at `.claude/skills/meta/SKILL.md`.

### What the AI Can Do

| Task | Natural Language | Command |
|------|------------------|---------|
| Check status | "What's running?" | `meta status` |
| Start dev | "Start the dev servers" | `meta dev` |
| Stop dev | "Stop everything" | `meta dev:stop` |
| Debug | "Why is the API down?" | `meta status` + `meta logs api` |

### Invoking

```
> /meta
> /meta status
> "Check if my servers are running"
```

## Tmux Navigation

All tmux commands start with **prefix**: `Ctrl+B`

### Essential Shortcuts

| Action | Keys |
|--------|------|
| Move right | `Ctrl+B` → `→` |
| Move left | `Ctrl+B` → `←` |
| Move up | `Ctrl+B` → `↑` |
| Move down | `Ctrl+B` → `↓` |
| Zoom toggle | `Ctrl+B` → `Z` |
| Show numbers | `Ctrl+B` → `Q` |
| Detach | `Ctrl+B` → `D` |
| Kill pane | `Ctrl+B` → `X` |

### Advanced

```bash
tmux ls                    # List sessions
tmux attach -t <name>      # Attach to session
```

## Recording Demos

```bash
cd docs/launch
./record-demo.sh      # Record with asciinema
./convert-to-gif.sh   # Convert to GIF
```

See [DEMO_SCRIPT.md](../../../docs/launch/DEMO_SCRIPT.md) for recording guide.

## Troubleshooting

### Tmux Session Won't Start

**Error:** `open terminal failed: not a terminal`

**Fix:** Run in a real terminal, not IDE or automation.

### Turbo Panes Exit Immediately

**Causes:**
- Missing `run` keyword in command
- Wrong package name in `--filter`

**Fix:** Run `meta doctor` to validate.

### Session Already Exists

```bash
meta sessions              # List sessions
tmux attach -t meta-<name> # Attach to existing
# or
meta dev:stop              # Stop and restart
meta dev
```

### Bacon Not Found

```bash
cargo install bacon
meta doctor
```

### Can't Navigate Panes

Use the **prefix** first:
1. Press `Ctrl+B`
2. Release
3. Press navigation key

## Getting Help

```bash
meta --help
meta <command> --help
meta doctor
```

**More docs:**
- [README](../README.md) - Features and CLI reference
- [Standalone Setup](../STANDALONE.md) - Add to your monorepo
