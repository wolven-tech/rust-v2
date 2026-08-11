# Contributing to Meta

Thank you for your interest in contributing!

## Setup

```bash
git clone https://github.com/wolven-tech/rust-v1.git
cd rust-v1/tooling/meta
cargo build
cargo test
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature
# or
git checkout -b fix/bug-description
```

### 2. Make Changes

- Follow Rust idioms
- Add tests for new features
- Update docs if needed

### 3. Run Quality Checks

```bash
cargo fmt           # Format code
cargo clippy        # Lint (zero warnings required)
cargo test          # Run tests
cargo audit         # Security check
```

### 4. Commit

```bash
git commit -m "feat: add awesome feature"
```

**Commit prefixes:** `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`

### 5. Create PR

Include:
- Summary of changes
- Motivation
- How it was tested

## Code Standards

- **Zero warnings** - Clippy warnings are errors
- **Tests required** - New features need tests
- **Documentation** - Public APIs need doc comments

## Architecture

```
src/
├── main.rs           # Entry point
├── cli.rs            # CLI parsing (clap)
├── config.rs         # meta.toml loading
├── execution/        # Task execution & tmux
└── adapters/         # Tool adapters
```

## Testing

```bash
cargo test                          # All tests
cargo test test_name                # Specific test
cargo test test_name -- --nocapture # With output
```

## Common Issues

**Clippy warnings:**
```bash
cargo clippy --fix --allow-dirty
```

**Formatting:**
```bash
cargo fmt --all
```

## CI Pipeline

GitHub Actions runs on every PR:
- Tests (Ubuntu, macOS, Windows)
- Formatting check
- Clippy linting
- Security audit
