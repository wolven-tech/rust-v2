# Using Meta in Your Monorepo

This guide shows how to add **meta** to your own monorepo project.

## Prerequisites

- **tmux** - Required for multi-pane orchestration
- **Rust & Cargo** - For building meta
- **Bacon** (optional) - For Rust hot-reload
- **Turborepo** (optional) - For TypeScript/Next.js projects

## Installation

### Option 1: From crates.io (Recommended)

```bash
cargo install monorepo-meta
```

### Option 2: From Source

```bash
git clone https://github.com/wolven-tech/rust-v1.git
cd rust-v1/tooling/meta
cargo install --path .
```

### Option 3: Copy to Your Monorepo

The `tooling/meta` directory is self-contained:

```bash
cp -r path/to/rust-v1/tooling/meta your-monorepo/tooling/meta
cd your-monorepo/tooling/meta
cargo install --path .
```

## Setup

### 1. Initialize Configuration

```bash
cd your-monorepo/
meta init
```

This auto-detects projects in `apps/` and `packages/` and generates `meta.toml`.

### 2. Customize meta.toml

Edit `meta.toml` to match your project structure:

```toml
version = "1"

[workspace]
name = "My Monorepo"

[tools.bacon]
enabled = true
command = "bacon"
for_languages = ["rust"]

[tools.turborepo]
enabled = true
command = "turbo"
for_languages = ["typescript"]

[projects.api]
type = "rust"
path = "services/api"

[projects.api.tasks]
dev = { tool = "bacon", command = "run-long" }
build = { tool = "cargo", command = "build --release" }

[projects.web]
type = "next"
path = "apps/web"

[projects.web.tasks]
dev = { tool = "turborepo", command = "run dev --filter=@your-org/web" }
```

### 3. Configure Bacon Logging (Optional)

For bacon projects to write logs, update your `bacon.toml`:

```toml
[jobs.run-long]
command = ["sh", "-c", "mkdir -p ../../.meta/logs && cargo run --color always 2>&1 | tee ../../.meta/logs/api.log"]
need_stdout = true
```

### 4. Validate

```bash
meta doctor
```

### 5. Start Development

```bash
meta dev
```

## Integration with package.json

Add meta commands to your root `package.json`:

```json
{
  "scripts": {
    "dev": "meta dev",
    "build": "meta build",
    "doctor": "meta doctor"
  }
}
```

## Updating Meta

```bash
cargo install monorepo-meta
```

## More Information

- **[README](README.md)** - Full feature list and CLI reference
- **[User Guide](docs/USER_GUIDE.md)** - Daily workflow and tmux navigation
